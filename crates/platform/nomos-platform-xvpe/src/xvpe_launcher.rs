use nomos_platform::{Command, ExitOutcome, ProgramLauncher};
use xvpe_primitives::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use xvpe_subprocess_execution::{
    EnvironmentEdit, ExitOutcome as XvpeExitOutcome, ProcessCommand, ProcessLauncherStrategy,
    ProcessOutput as XvpeProcessOutput,
};

/// Why a command carrying environment edits is refused rather than run.
const ENVIRONMENT_NOT_HONOURED: &str = "this launcher cannot honour environment edits, because a nomos Command carries no environment, so it refuses the command rather than run it without them; the edits name";

/// Any of this workspace's launchers, seen as the launcher surface XVPE's own
/// adapters take.
///
/// # Why a bridge rather than a replacement
///
/// XVPE's port and this one are the same design — they were the same code, until
/// it moved down there — but the trait differs in one respect: XVPE requires
/// every strategy surface to declare its determinism. Adopting that trait here
/// would mean touching **33 implementors**, nearly all of them test doubles, and
/// **559 use-sites** across twenty crates, for no behavioural change at all.
///
/// One adapter costs none of that. Everything in this workspace keeps naming its
/// own port; anything reaching into XVPE wraps its launcher once, here.
pub struct XvpeLauncher<'launcher, Launcher: ProgramLauncher>
{
    launcher: &'launcher Launcher,
}

impl<'launcher, Launcher: ProgramLauncher> XvpeLauncher<'launcher, Launcher>
{
    /// See this launcher as XVPE's.
    #[must_use]
    pub const fn Wrapping(launcher: &'launcher Launcher) -> Self
    {
        return Self { launcher };
    }
}

impl<Launcher: ProgramLauncher> Strategy for XvpeLauncher<'_, Launcher>
{
    // Whatever the wrapped launcher promises, this adapter adds nothing and
    // claims nothing: it translates two structurally identical descriptions of
    // the same command.
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

impl<Launcher: ProgramLauncher> ProcessLauncherStrategy for XvpeLauncher<'_, Launcher>
{
    fn Run(&self, command: &ProcessCommand) -> Result<XvpeProcessOutput, String>
    {
        // Refused, not dropped. A nomos Command has nowhere to put an environment edit, and
        // running the command without one is not a smaller version of the same request: the
        // DeepSeek backend's edits remove ANTHROPIC_API_KEY and point the CLI elsewhere, so the
        // same dispatch run without them reaches the wrong provider on the caller's own key and
        // reports success.
        if !command.environment_edits.is_empty()
        {
            return Err(Unhonoured(&command.environment_edits));
        }

        let translated = Command {
            argv: command.argv.clone(),
            working_directory: command.working_directory.clone(),
            timeout: command.timeout,
            idle_timeout: command.idle_timeout,
        };

        let output = self.launcher.Run(&translated)?;

        return Ok(XvpeProcessOutput {
            outcome: Outcome_For(output.outcome),
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }
}

/// The refusal for a command whose environment edits this launcher cannot honour.
///
/// It names each variable and never a value: a value set here can be a credential, and a
/// refusal is exactly the text that ends up in a log.
fn Unhonoured(edits: &[EnvironmentEdit]) -> String
{
    let names: Vec<&str> = edits
        .iter()
        .map(|edit| {
            return match edit
            {
                EnvironmentEdit::Set(name, _) | EnvironmentEdit::Remove(name) => name.as_str(),
            };
        })
        .collect();

    return format!("{ENVIRONMENT_NOT_HONOURED} {}", names.join(", "));
}

/// One outcome vocabulary in the other.
///
/// Field for field, because they are the same distinctions: a clean exit, a wall
/// bound, an idle bound, and an abnormal end. If either side ever grows a
/// variant the other lacks, this function is where that shows up as a
/// compilation failure rather than as a silently collapsed case.
fn Outcome_For(outcome: ExitOutcome) -> XvpeExitOutcome
{
    return match outcome
    {
        ExitOutcome::Exited { code } => XvpeExitOutcome::Exited { code },
        ExitOutcome::TimedOut => XvpeExitOutcome::TimedOut,
        ExitOutcome::Stalled { idle_elapsed } => XvpeExitOutcome::Stalled { idle_elapsed },
        ExitOutcome::Terminated => XvpeExitOutcome::Terminated,
    };
}

#[cfg(test)]
mod tests
{
    use std::cell::RefCell;
    use std::time::Duration;

    use nomos_platform::ProgramOutput;

    use super::*;

    /// An arbitrary bound for commands whose timing is not under test.
    const A_BOUND: Duration = Duration::from_secs(30);
    /// A short idle bound, distinct from [`A_BOUND`] so that a translation which dropped the
    /// idle bound would be visible as a difference rather than as an equal pair.
    const A_SHORT_BOUND: Duration = Duration::from_secs(5);
    /// How long an idle clock is said to have run, and a failing exit code to have carried.
    /// Neither is judged here; both are carried through the translation unchanged.
    const AN_IDLE_ELAPSED: Duration = Duration::from_secs(7);
    const A_FAILING_EXIT_CODE: i32 = 3;
    /// What the wrapped launcher says back.
    const SAID: &str = "said something";

    /// The command must arrive at the wrapped launcher unchanged.
    const COMMAND_MUST_SURVIVE: &str = "the command reaches the wrapped launcher unchanged";
    /// Every outcome must map, and none may collapse into another.
    const OUTCOMES_MUST_MAP: &str = "each outcome maps to its own counterpart";

    struct Recording
    {
        seen: RefCell<Vec<Command>>,
        outcome: ExitOutcome,
    }

    impl Recording
    {
        fn Reporting(outcome: ExitOutcome) -> Self
        {
            return Self { seen: RefCell::new(Vec::new()), outcome };
        }
    }

    // Answers from fixed data, so its outputs reproduce byte for byte. Written through
    // paths rather than imports: this file is the bridge, so both workspaces'
    // determinism vocabularies are in scope and every name in them collides.
    impl nomos_platform::Strategy for Recording
    {
        const STRENGTH: nomos_platform::DeterminismStrength = nomos_platform::DeterminismStrength::State;
        const SCOPE: nomos_platform::ReproducibilityScope = nomos_platform::ReproducibilityScope::SingleRun;
        const TRACE: nomos_platform::TraceEquivalence = nomos_platform::TraceEquivalence::BitIdentical;
    }

    impl ProgramLauncher for Recording
    {
        fn Run(&self, command: &Command) -> Result<ProgramOutput, String>
        {
            self.seen.borrow_mut().push(command.clone());
            return Ok(ProgramOutput {
                outcome: self.outcome,
                stdout: SAID.to_owned(),
                stderr: String::new(),
            });
        }
    }

    #[test]
    fn Test_The_Command_Should_Reach_The_Wrapped_Launcher_Unchanged()
    {
        let inner = Recording::Reporting(ExitOutcome::Exited { code: 0 });
        let command = ProcessCommand::New(vec!["git".to_owned(), "status".to_owned()], A_BOUND)
            .With_Idle_Timeout(A_SHORT_BOUND);

        let output = XvpeLauncher::Wrapping(&inner).Run(&command).expect("the wrapped launcher answers, and this double answers every command");

        let seen = inner.seen.borrow();
        let seen = seen.first().expect("the double recorded exactly one command before answering");
        assert_eq!(seen.argv, command.argv, "{COMMAND_MUST_SURVIVE}");
        assert_eq!(seen.timeout, command.timeout, "{COMMAND_MUST_SURVIVE}");
        // The idle bound is the one a translation is most likely to drop, since
        // a caller that never asks for it never notices it is gone.
        assert_eq!(seen.idle_timeout, command.idle_timeout, "{COMMAND_MUST_SURVIVE}");
        assert_eq!(output.stdout, SAID, "{COMMAND_MUST_SURVIVE}");
    }

    /// A variable an edit names, and a value that must never appear in a refusal.
    const REMOVED_VARIABLE: &str = "ANTHROPIC_API_KEY";
    const SET_VARIABLE: &str = "ANTHROPIC_AUTH_TOKEN";
    const SECRET_VALUE: &str = "a credential nobody should read in a log";

    /// A command whose edits cannot be honoured must not reach the process at all.
    const EDITS_MUST_REFUSE: &str = "a command with environment edits was run without them";
    /// A refusal names what it could not honour, and never a value.
    const REFUSAL_MUST_NAME_NOT_REVEAL: &str = "the refusal names each variable and no value";

    #[test]
    fn Test_A_Command_With_Environment_Edits_Should_Be_Refused_Not_Run_Without_Them()
    {
        let inner = Recording::Reporting(ExitOutcome::Exited { code: 0 });
        let command = ProcessCommand::New(vec!["claude".to_owned()], A_BOUND)
            .Removing_Environment_Variable(REMOVED_VARIABLE.to_owned())
            .With_Environment_Variable(SET_VARIABLE.to_owned(), SECRET_VALUE.to_owned());

        let refusal = XvpeLauncher::Wrapping(&inner)
            .Run(&command)
            .expect_err(EDITS_MUST_REFUSE);

        assert!(inner.seen.borrow().is_empty(), "{EDITS_MUST_REFUSE}");
        assert!(refusal.contains(REMOVED_VARIABLE), "{REFUSAL_MUST_NAME_NOT_REVEAL}");
        assert!(refusal.contains(SET_VARIABLE), "{REFUSAL_MUST_NAME_NOT_REVEAL}");
        assert!(!refusal.contains(SECRET_VALUE), "{REFUSAL_MUST_NAME_NOT_REVEAL}");
    }

    #[test]
    fn Test_Every_Outcome_Should_Map_To_Its_Own_Counterpart()
    {
        let idle_elapsed = AN_IDLE_ELAPSED;
        for (theirs, ours) in [
            (
                ExitOutcome::Exited { code: A_FAILING_EXIT_CODE },
                XvpeExitOutcome::Exited { code: A_FAILING_EXIT_CODE },
            ),
            (ExitOutcome::TimedOut, XvpeExitOutcome::TimedOut),
            (ExitOutcome::Stalled { idle_elapsed }, XvpeExitOutcome::Stalled { idle_elapsed }),
            (ExitOutcome::Terminated, XvpeExitOutcome::Terminated),
        ]
        {
            assert_eq!(Outcome_For(theirs), ours, "{OUTCOMES_MUST_MAP}");
        }
    }
}
