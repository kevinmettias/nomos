//! Running the shared gate steps the predicate does not cover.

use super::{FileSystem, Clock, FilesystemLock, FileLedger, ItemId, Path, FinishRefusal, Workflow_Path, GateUnknown, Derive_Step, LINT_STEP, RULES_STEP, OptionalStepOutcome, StepName, WorkflowText, ProgramLauncher, Runner, GateOutcome, Command_From_Argv, Ran_To_Completion};

/// What the gate's own steps answered, before the item's predicate was asked.
pub(super) struct GateSteps
{
    /// The `Lint` step, which every workflow a finish accepts declares.
    pub(super) lint: GateOutcome,
    /// The `Rules` step, or the fact that the workflow declares none.
    pub(super) rules: OptionalStepOutcome,
}

/// The commands a finish runs ahead of the predicate, each read out of the workflow rather
/// than written here.
struct GateArgvs
{
    lint: Vec<String>,
    /// `None` when the workflow declares no `Rules` step, which `OD-GATE-036` allows.
    rules: Option<Vec<String>>,
}

/// Both steps' commands, derived from one reading of the workflow before either runs.
///
/// Every failure is `GateUndetermined` rather than a licence to run the predicate alone: a
/// workflow nobody could read, one with no lint step, and one whose step is a script all
/// leave the question "would this land" unanswered, and that is not the same as answering it
/// yes. Both are derived before either runs, so an underivable `Rules` step refuses before the
/// lint step has spent its minutes and nothing runs at all -- `OD-LEDGER-003`'s rule for an
/// undetermined gate, carried to the second step.
fn Gate_Argvs<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    item: &ItemId,
    working_directory: Option<&Path>,
) -> Result<GateArgvs, FinishRefusal>
{
    let workflow = Workflow_Text(ledger, item, working_directory)?;
    let undetermined = |cause| return FinishRefusal::GateUndetermined { item: item.clone(), cause };

    let lint = Derive_Step(&workflow, LINT_STEP).map_err(undetermined)?;
    let rules = Declared_Step(WorkflowText::from(&workflow), StepName::from(RULES_STEP)).map_err(undetermined)?;

    return Ok(GateArgvs { lint, rules });
}

/// The gate workflow's text, or a refusal naming where it was looked for.
fn Workflow_Text<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    item: &ItemId,
    working_directory: Option<&Path>,
) -> Result<String, FinishRefusal>
{
    let workflow_path = Workflow_Path(working_directory.unwrap_or_else(|| return Path::new(".")));

    return ledger.Read_File(&workflow_path).map_err(|cause| {
        return FinishRefusal::GateUndetermined {
            item: item.clone(),
            cause: GateUnknown::Unreadable {
                path: workflow_path.display().to_string(),
                cause,
            },
        };
    });
}

/// A step a workflow is free not to declare: its command, or `None` when no step by that
/// name exists.
///
/// Only [`GateUnknown::NoSuchStep`] is absence. Every other cause -- a scripted `run:`, or a
/// step declared with no command at all -- is a step that is there and cannot be derived, and
/// taking it as absent would record a step nobody ran as one nobody declared.
fn Declared_Step(workflow: WorkflowText<'_>, step: StepName<'_>) -> Result<Option<Vec<String>>, GateUnknown>
{
    return match Derive_Step(workflow, step)
    {
        Ok(argv) => Ok(Some(argv)),
        Err(GateUnknown::NoSuchStep { .. }) => Ok(None),
        Err(cause) => Err(cause),
    };
}

/// Runs the gate's `Lint` step and then its `Rules` step, each derived from the workflow
/// rather than written here.
///
/// Separate from [`super::Finish_Item`] because it is a separate question. That function
/// asks whether the item's own predicate holds; this asks whether the work can land at all,
/// and the second question is the one every item's predicate was silently skipping. The lint
/// step answers it for what the compiler denies, and the `Rules` step for what the rule layer
/// blocks, which no predicate sized from its own packages reaches -- `OD-GATE-036`.
///
/// The first step to exit nonzero ends the finish, so the `Rules` step does not run behind a
/// red lint step and the predicate runs behind neither.
///
/// # Errors
///
/// Returns [`FinishRefusal::GateUndetermined`] when what the gate checks cannot be
/// established — which is a refusal, not a licence to run the predicate alone — and
/// [`FinishRefusal::GateFailed`] when a step ran and the answer was no.
pub(super) fn Run_Gate_Steps<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    launcher: &impl ProgramLauncher,
    item: &ItemId,
    runner: Runner<'_>,
) -> Result<GateSteps, FinishRefusal>
{
    let argvs = Gate_Argvs(ledger, item, runner.working_directory)?;
    let lint = Run_Derived_Step(launcher, item, runner, argvs.lint)?;
    let rules = match argvs.rules
    {
        Some(argv) => OptionalStepOutcome::Ran(Run_Derived_Step(launcher, item, runner, argv)?),
        None => OptionalStepOutcome::NotDeclared,
    };

    return Ok(GateSteps { lint, rules });
}

/// One derived step, run under the item's own bound and refused unless it exits zero.
///
/// Both steps go through this, so the `Rules` step is bounded by exactly the patience the
/// lint step is. Any nonzero exit -- a Blocking finding's 1, a run that judged nothing's 6, a
/// run that could not be assembled's 5 -- refuses as [`FinishRefusal::GateFailed`], carrying
/// the step's argv and the tail of what it printed. Zero is the only success, which
/// `OD-GATE-004` decided for the step and nothing here relaxes.
fn Run_Derived_Step(
    launcher: &impl ProgramLauncher,
    item: &ItemId,
    runner: Runner<'_>,
    argv: Vec<String>,
) -> Result<GateOutcome, FinishRefusal>
{
    let command = Command_From_Argv(argv.clone(), runner);
    let ran = Ran_To_Completion(launcher, &command, item)?;

    if ran.code != 0
    {
        return Err(FinishRefusal::GateFailed {
            item: item.clone(),
            argv,
            exit_code: ran.code,
            output_tail: ran.tail,
        });
    }

    return Ok(GateOutcome {
        argv,
        exit_code: ran.code,
    });
}

#[cfg(test)]
mod tests
{
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
    use super::*;
    use nomos_platform::{Command, ExitOutcome, ProgramOutput, Timestamp};
    use nomos_platform_std::{FileLock, StdFileSystem};

    struct FixedClock(i64);

    /// Fixed instants, so both the values and their timing reproduce.
    impl Strategy for FixedClock
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl Clock for &FixedClock
    {
        fn Now(&self) -> Timestamp
        {
            return Timestamp::From_Unix_Seconds(self.0);
        }
    }

    const WORKFLOW: &str = "name: gate\n\
                            \n\
                            jobs:\n\
                            \x20 gate:\n\
                            \x20   steps:\n\
                            \x20     - uses: actions/checkout@v4\n\
                            \x20     - name: Lint\n\
                            \x20       run: cargo clippy --workspace --all-targets -- -D warnings\n\
                            \x20     - name: Test\n\
                            \x20       run: cargo test --workspace\n\
                            \x20     - name: Rules\n\
                            \x20       run: cargo run --quiet -p nomos-cli --bin nomos -- gate run --root .\n";

    /// The exit code a lint step reports when it found problems. It is the code [`WORKFLOW`]'s
    /// Lint step turns into a refusal, so the fixture's launcher and the case's expectation
    /// read the same number.
    const LINT_EXIT_CODE: i32 = 101;

    /// The instant the gate cases ask at. Any fixed second does; what they need is that it
    /// does not move between runs.
    const NOW_SECONDS: i64 = 1_000;

    /// How long the runner lets the lint step take. Long enough that no case here is a race
    /// against the bound rather than a case about the gate.
    const GATE_TIMEOUT_SECONDS: u64 = 60;

    /// A launcher standing in for a lint step that finds a problem.
    struct AlwaysFails;

    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for AlwaysFails
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl ProgramLauncher for &AlwaysFails
    {
        fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
        {
            return Ok(ProgramOutput {
                outcome: ExitOutcome::Exited { code: LINT_EXIT_CODE },
                stdout: String::new(),
                stderr: "clippy found problems".to_owned(),
            });
        }
    }

    #[test]
    fn Test_Gate_Argvs_Should_Derive_The_Lint_Steps_Command_From_The_Workflow()
    {
        let directory = Tree_With_Workflow("gate-argv", WORKFLOW);
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);
        let item = ItemId::New("G-1");

        let argvs = Gate_Argvs(&ledger, &item, Some(&directory)).unwrap_or_else(|refusal| panic!("{}", refusal.Describe()));

        assert_eq!(argvs.lint, Words("cargo clippy --workspace --all-targets -- -D warnings"));
    }

    /// The `Rules` step comes out of the same reading of the same file, by its own name.
    #[test]
    fn Test_Gate_Argvs_Should_Derive_The_Rules_Steps_Command_Beside_The_Lint_Step()
    {
        let directory = Tree_With_Workflow("gate-argv-rules", WORKFLOW);
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);
        let item = ItemId::New("G-1R");

        let argvs = Gate_Argvs(&ledger, &item, Some(&directory)).unwrap_or_else(|refusal| panic!("{}", refusal.Describe()));

        assert_eq!(argvs.rules, Some(Words("cargo run --quiet -p nomos-cli --bin nomos -- gate run --root .")));
    }

    #[test]
    fn Test_Gate_Argvs_Should_Report_An_Undetermined_Gate_When_No_Workflow_Exists()
    {
        let directory = Temporary_Directory("gate-argv-missing");
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);
        let item = ItemId::New("G-1B");

        let Err(refusal) = Gate_Argvs(&ledger, &item, Some(&directory))
        else
        {
            panic!("no workflow means no derivable step");
        };

        assert!(
            matches!(
                refusal,
                FinishRefusal::GateUndetermined { cause: GateUnknown::Unreadable { .. }, .. }
            ),
            "got {refusal:?}"
        );
    }

    /// Absence is one cause and only one. A workflow without the step yields `None`; a step
    /// that is there and cannot be derived -- a script, or no `run:` line at all -- is a
    /// refusal, because recording it as absent would claim nobody declared a step somebody did.
    #[test]
    fn Test_Declared_Step_Should_Take_Only_An_Undeclared_Step_As_Absent()
    {
        let without = WORKFLOW.replace("- name: Rules", "- name: Something else");
        let as_action = WORKFLOW.replace(
            "run: cargo run --quiet -p nomos-cli --bin nomos -- gate run --root .",
            "uses: some/rules-action@v1",
        );
        let scripted = WORKFLOW.replace("--root .", "--root . || true");

        assert_eq!(Declared_Step(WorkflowText::from(&without), StepName::from(RULES_STEP)), Ok(None));
        for underivable in [as_action, scripted]
        {
            let answer = Declared_Step(WorkflowText::from(&underivable), StepName::from(RULES_STEP));

            assert!(matches!(answer, Err(GateUnknown::NotASingleCommand { .. })), "got {answer:?}");
        }
    }

    #[test]
    fn Test_Run_Gate_Steps_Should_Refuse_When_The_Lint_Command_Exits_Nonzero()
    {
        let directory = Tree_With_Workflow("run-gate-step", WORKFLOW);
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);
        let item = ItemId::New("G-2");
        let runner = Runner {
            working_directory: Some(directory.as_path()),
            timeout: std::time::Duration::from_secs(GATE_TIMEOUT_SECONDS),
        };
        let launcher = AlwaysFails;

        let Err(refusal) = Run_Gate_Steps(&ledger, &&launcher, &item, runner)
        else
        {
            panic!("a nonzero lint exit must refuse");
        };

        assert!(
            matches!(refusal, FinishRefusal::GateFailed { exit_code: LINT_EXIT_CODE, .. }),
            "got {refusal:?}"
        );
    }

    /// A command line as the argv a bare `run:` line splits into.
    fn Words(line: &str) -> Vec<String>
    {
        return line.split_whitespace().map(str::to_owned).collect();
    }

    fn Temporary_Directory(name: &str) -> std::path::PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-gate-step-{name}-{}", std::process::id()));
        if path.exists()
        {
            std::fs::remove_dir_all(&path).expect("the previous run's synthetic directory is removable");
        }
        std::fs::create_dir_all(&path).expect("test needs a temp directory");
        return path;
    }

    fn Tree_With_Workflow(name: &str, workflow: &str) -> std::path::PathBuf
    {
        let directory = Temporary_Directory(name);
        let workflows = directory.join(".github").join("workflows");
        std::fs::create_dir_all(&workflows).expect("test needs a workflow directory");
        std::fs::write(workflows.join("gate.yml"), workflow).expect("test needs a workflow");
        return directory;
    }

    fn Ledger_At<'clock>(
        directory: &Path,
        clock: &'clock FixedClock,
    ) -> FileLedger<StdFileSystem, &'clock FixedClock, FileLock>
    {
        return FileLedger::At(
            directory.join("ledger.json"),
            StdFileSystem,
            clock,
            FileLock::At(directory.join("ledger.lock")),
        );
    }
}
