//! Running the shared gate step the predicate does not cover.

use super::{FileSystem, Clock, FilesystemLock, FileLedger, ItemId, Path, FinishRefusal, Workflow_Path, GateUnknown, Derive_Step, LINT_STEP, ProgramLauncher, Runner, GateOutcome, Command_From_Argv, Ran_To_Completion};

/// The gate's lint step, read out of the workflow rather than written here.
///
/// Both failures are `GateUndetermined` rather than a licence to run the predicate alone:
/// a workflow nobody could read and a workflow with no such step both leave the question
/// "would this land" unanswered, and that is not the same as answering it yes.
pub(super) fn Gate_Argv<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    item: &ItemId,
    working_directory: Option<&Path>,
) -> Result<Vec<String>, FinishRefusal>
{
    let workflow_path = Workflow_Path(working_directory.unwrap_or_else(|| return Path::new(".")));
    let workflow = ledger
        .Read_File(&workflow_path)
        .map_err(|cause| FinishRefusal::GateUndetermined {
            item: item.clone(),
            cause: GateUnknown::Unreadable {
                path: workflow_path.display().to_string(),
                cause,
            },
        })?;

    return Derive_Step(&workflow, LINT_STEP).map_err(|cause| {
        return FinishRefusal::GateUndetermined {
            item: item.clone(),
            cause,
        };
    });
}

/// Runs the gate's lint step, derived from the workflow rather than written here.
///
/// Separate from [`Finish`] because it is a separate question. [`Finish`] asks whether the
/// item's own predicate holds; this asks whether the work can land at all, and the second
/// question is the one every item's predicate was silently skipping.
///
/// # Errors
///
/// Returns [`FinishRefusal::GateUndetermined`] when what the gate checks cannot be
/// established — which is a refusal, not a licence to run the predicate alone — and
/// [`FinishRefusal::GateFailed`] when the step ran and the answer was no.
pub(super) fn Run_Gate_Step<Files: FileSystem, TimeSource: Clock, Lock: FilesystemLock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    launcher: &impl ProgramLauncher,
    item: &ItemId,
    runner: Runner<'_>,
) -> Result<GateOutcome, FinishRefusal>
{
    let argv = Gate_Argv(ledger, item, runner.working_directory)?;
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
                            \x20       run: cargo test --workspace\n";

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
    fn Test_Gate_Argv_Should_Derive_The_Lint_Steps_Command_From_The_Workflow()
    {
        let directory = Tree_With_Workflow("gate-argv");
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);
        let item = ItemId::New("G-1");

        let argv = Gate_Argv(&ledger, &item, Some(&directory)).expect("the workflow declares a Lint step");

        assert_eq!(
            argv,
            vec!["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn Test_Gate_Argv_Should_Report_An_Undetermined_Gate_When_No_Workflow_Exists()
    {
        let directory = Temporary_Directory("gate-argv-missing");
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);
        let item = ItemId::New("G-1B");

        let refusal = Gate_Argv(&ledger, &item, Some(&directory)).expect_err("no workflow means no derivable step");

        assert!(
            matches!(
                refusal,
                FinishRefusal::GateUndetermined { cause: GateUnknown::Unreadable { .. }, .. }
            ),
            "got {refusal:?}"
        );
    }

    #[test]
    fn Test_Run_Gate_Step_Should_Refuse_When_The_Lint_Command_Exits_Nonzero()
    {
        let directory = Tree_With_Workflow("run-gate-step");
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);
        let item = ItemId::New("G-2");
        let runner = Runner {
            working_directory: Some(directory.as_path()),
            timeout: std::time::Duration::from_secs(GATE_TIMEOUT_SECONDS),
        };
        let launcher = AlwaysFails;

        let refusal = Run_Gate_Step(&ledger, &&launcher, &item, runner).expect_err("a nonzero lint exit must refuse");

        assert!(
            matches!(refusal, FinishRefusal::GateFailed { exit_code: LINT_EXIT_CODE, .. }),
            "got {refusal:?}"
        );
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

    fn Tree_With_Workflow(name: &str) -> std::path::PathBuf
    {
        let directory = Temporary_Directory(name);
        let workflows = directory.join(".github").join("workflows");
        std::fs::create_dir_all(&workflows).expect("test needs a workflow directory");
        std::fs::write(workflows.join("gate.yml"), WORKFLOW).expect("test needs a workflow");
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
