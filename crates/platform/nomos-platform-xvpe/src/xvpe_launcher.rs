use nomos_platform::{Command, ExitOutcome, ProcessLauncher};
use xvpe_primitives::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use xvpe_subprocess_execution::{
    ExitOutcome as XvpeExitOutcome, ProcessCommand, ProcessLauncherStrategy,
    ProcessOutput as XvpeProcessOutput,
};

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
pub struct XvpeLauncher<'launcher, Launcher: ProcessLauncher>
{
    launcher: &'launcher Launcher,
}

impl<'launcher, Launcher: ProcessLauncher> XvpeLauncher<'launcher, Launcher>
{
    /// See this launcher as XVPE's.
    #[must_use]
    pub const fn Wrapping(launcher: &'launcher Launcher) -> Self
    {
        return Self { launcher };
    }
}

impl<Launcher: ProcessLauncher> Strategy for XvpeLauncher<'_, Launcher>
{
    // Whatever the wrapped launcher promises, this adapter adds nothing and
    // claims nothing: it translates two structurally identical descriptions of
    // the same command.
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

impl<Launcher: ProcessLauncher> ProcessLauncherStrategy for XvpeLauncher<'_, Launcher>
{
    fn Run(&self, command: &ProcessCommand) -> Result<XvpeProcessOutput, String>
    {
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

    use nomos_platform::ProcessOutput;

    use super::*;

    /// An arbitrary bound for commands whose timing is not under test.
    const A_BOUND: Duration = Duration::from_secs(30);
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

    impl ProcessLauncher for Recording
    {
        fn Run(&self, command: &Command) -> Result<ProcessOutput, String>
        {
            self.seen.borrow_mut().push(command.clone());
            return Ok(ProcessOutput {
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
            .With_Idle_Timeout(Duration::from_secs(5));

        let output = XvpeLauncher::Wrapping(&inner).Run(&command).expect(COMMAND_MUST_SURVIVE);

        let seen = inner.seen.borrow();
        let seen = seen.first().expect(COMMAND_MUST_SURVIVE);
        assert_eq!(seen.argv, command.argv, "{COMMAND_MUST_SURVIVE}");
        assert_eq!(seen.timeout, command.timeout, "{COMMAND_MUST_SURVIVE}");
        // The idle bound is the one a translation is most likely to drop, since
        // a caller that never asks for it never notices it is gone.
        assert_eq!(seen.idle_timeout, command.idle_timeout, "{COMMAND_MUST_SURVIVE}");
        assert_eq!(output.stdout, SAID, "{COMMAND_MUST_SURVIVE}");
    }

    #[test]
    fn Test_Every_Outcome_Should_Map_To_Its_Own_Counterpart()
    {
        let idle_elapsed = Duration::from_secs(7);
        for (theirs, ours) in [
            (ExitOutcome::Exited { code: 3 }, XvpeExitOutcome::Exited { code: 3 }),
            (ExitOutcome::TimedOut, XvpeExitOutcome::TimedOut),
            (ExitOutcome::Stalled { idle_elapsed }, XvpeExitOutcome::Stalled { idle_elapsed }),
            (ExitOutcome::Terminated, XvpeExitOutcome::Terminated),
        ]
        {
            assert_eq!(Outcome_For(theirs), ours, "{OUTCOMES_MUST_MAP}");
        }
    }
}
