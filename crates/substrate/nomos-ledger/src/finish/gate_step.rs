//! Running the shared gate steps the predicate does not cover.

use super::{FileSystem, Clock, FilesystemLock, FileLedger, ItemId, Path, FinishRefusal, Workflow_Path, GateUnknown, Derive_Step, LINT_STEP, RULES_STEP, OptionalStepOutcome, StepName, WorkflowText, ProgramLauncher, Runner, GateOutcome, Command_From_Argv};
use super::running::{Combined_Tail, Exited_With_A_Code};
use nomos_platform::ProgramOutput;

/// How a line of `nomos gate run`'s report begins when the finding on it is in the Blocking
/// gate category: `Finding::Describe` puts `GateCategory::Label` in brackets at the head of
/// every finding line it renders.
///
/// This is matching rendered text, which is the last resort, and it is here because no
/// structured signal says which finding blocked:
///
/// - The finish runs the workflow's own command, derived and never copied (`OD-LEDGER-003`),
///   and in this repository that command prints a report for people: one line per finding on
///   standard output and nothing machine-readable beside it. Its machine-readable form, the
///   SARIF log, sits behind a flag the workflow does not pass, and a finish that added it would
///   be running a command the gate does not.
/// - The streams are structured, and they are used: only standard output, the step's own
///   report, is searched, so nothing cargo prints on standard error is lifted. But they separate
///   the report from the toolchain, not one finding from another, and the report's own end is
///   no place to rely on. In the case that showed the gap, the report's last 2,000 bytes alone
///   would have held the Blocking line with 199 to spare, and the same tree judged without
///   `GOROOT` set reports two more advisory findings after it, 760 bytes, which push it out.
/// - Inside the report a finding's gate category exists only as this rendered label. This
///   crate does not depend on the one that owns it, and a finish should not have to, because
///   the same ledger serves repositories whose `Rules` step is not `nomos` at all.
///
/// A step that prints no such line, whether because it is not `nomos` or because the spelling
/// changed, has nothing lifted and refuses with exactly the tail it carried before. The label
/// names the category and not the disposition, so a Blocking finding a declared policy
/// tolerated still carries it; what is lifted is every line labelled Blocking, which holds every
/// finding that refused the run, and the refusal says "labelled" for that reason.
const BLOCKING_LINE_PREFIX: &str = "[Blocking] ";

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
/// the step's argv and what [`Refused_Step_Output`] keeps of what it printed. Zero is the only
/// success, which `OD-GATE-004` decided for the step and nothing here relaxes.
fn Run_Derived_Step(
    launcher: &impl ProgramLauncher,
    item: &ItemId,
    runner: Runner<'_>,
    argv: Vec<String>,
) -> Result<GateOutcome, FinishRefusal>
{
    let command = Command_From_Argv(argv.clone(), runner);
    let exited = Exited_With_A_Code(launcher, &command, item)?;

    if exited.code != 0
    {
        return Err(FinishRefusal::GateFailed {
            item: item.clone(),
            argv,
            exit_code: exited.code,
            output_tail: Refused_Step_Output(&exited.output),
        });
    }

    return Ok(GateOutcome {
        argv,
        exit_code: exited.code,
    });
}

/// What a refused step's refusal carries of what it printed: every line its report labelled
/// Blocking, lifted out of wherever it fell, and then the same tail every refusal carried
/// before.
///
/// The tail alone was not enough. It joins standard error after standard output, and `cargo run`
/// writes the toolchain's warnings on standard error, so they fill the tail first; and the
/// report does not put its Blocking findings last. The line that refused the finish sat outside
/// what the refusal kept, and its reader re-ran the whole gate to learn which finding it was.
/// [`BLOCKING_LINE_PREFIX`] says why the lines are found by their label.
///
/// The tail comes after the lifted lines, unchanged, so nothing a refusal showed before is
/// lost: a step that printed no such line -- a clippy lint step, or a `Rules` step that refused
/// for a reason other than a Blocking finding -- refuses with exactly the tail it always did.
fn Refused_Step_Output(output: &ProgramOutput) -> String
{
    let tail = Combined_Tail(output);
    let labelled: Vec<&str> =
        output.stdout.lines().filter(|line| return line.starts_with(BLOCKING_LINE_PREFIX)).collect();
    if labelled.is_empty()
    {
        return tail;
    }

    return format!(
        "every line its report labelled Blocking, lifted from wherever it fell in the report:\n{}\n\n\
         the end of what it printed:\n{tail}",
        labelled.join("\n")
    );
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

    /// The exit code `gate run` gives a run with a finding that can fail a build.
    const BLOCKING_EXIT_CODE: i32 = 1;

    /// The word that tells [`WORKFLOW`]'s lint command from its `Rules` command.
    const LINT_PROGRAM_WORD: &str = "clippy";

    /// The rule, the subject, the summary and the location of the Blocking finding
    /// `OD-GATE-036` reintroduced at `0000e0e3`, as `gate run` rendered it when this change
    /// reintroduced it again in a scratch copy of the tree.
    const BLOCKING_RULE: &str = "abbreviations";
    const BLOCKING_SUBJECT: &str = "Assert_Well_Formed_GraphML";
    const BLOCKING_SUMMARY: &str =
        "`Assert_Well_Formed_GraphML` contains the word `ml`, no vowels — an abbreviation; spell it out";
    const BLOCKING_LOCATION: &str = "crates/orchestration/nomos-gate-orchestration/src/export/supporting_fact_graph.rs";

    /// A second Blocking finding, so "every" is asked of more than one line.
    const SECOND_BLOCKING_LINE: &str =
        "[Blocking] single-letter-names: x (`x` is a single-letter name) — crates/substrate/nomos-ledger/src/holder.rs:12:9";

    /// How many Blocking lines [`Report_With_Blocking_Lines_Far_From_Its_End`] carries.
    const BLOCKING_LINES_IN_THE_REPORT: usize = 2;

    /// A line on standard error that begins as a Blocking report line does. Cargo's stream is
    /// the toolchain's and not the step's report, so nothing on it is lifted, however it begins.
    const TOOLCHAIN_LOOKALIKE_LINE: &str = "[Blocking] not-a-finding: printed by the toolchain on standard error";

    /// How many advisory findings follow each Blocking one in the report, and how many warnings
    /// cargo replays: enough that either group alone runs past the 2,000 bytes a refusal kept
    /// of everything together. The cases assert that of the text they build rather than trust
    /// this count for it.
    const LINES_OF_OTHER_OUTPUT: usize = 24;

    /// The Blocking finding as one rendered line, built from its parts so each part's presence
    /// in a refusal is asserted against the same text the report carried.
    fn Blocking_Line() -> String
    {
        return format!("[Blocking] {BLOCKING_RULE}: {BLOCKING_SUBJECT} ({BLOCKING_SUMMARY}) — {BLOCKING_LOCATION}");
    }

    /// Advisory findings as `gate run` renders them, numbered so no two are the same line.
    fn Advisory_Lines(first: usize) -> Vec<String>
    {
        return (first..first.saturating_add(LINES_OF_OTHER_OUTPUT))
            .map(|number| {
                return format!(
                    "[Advisory] lint-diagnostics: crates/substrate/nomos-ledger (warning [clippy::needless_pass_by_value]: \
                     this argument is passed by value) — crates/substrate/nomos-ledger/src/store/reservation.rs:{number}"
                );
            })
            .collect();
    }

    /// A report whose two Blocking lines each sit behind more than 2,000 bytes of advisory
    /// findings, closed by the summary line `gate run` prints last.
    fn Report_With_Blocking_Lines_Far_From_Its_End() -> String
    {
        let mut lines = vec!["run: 0123456789abcdef".to_owned(), Blocking_Line()];
        lines.extend(Advisory_Lines(0));
        lines.push(SECOND_BLOCKING_LINE.to_owned());
        lines.extend(Advisory_Lines(LINES_OF_OTHER_OUTPUT));
        lines.push(String::new());
        lines.push("50 finding(s), 2 of which can fail a build, 0 below the evidence floor, 0 calibrated, 0 suppressed, 0 baselined".to_owned());

        return lines.join("\n") + "\n";
    }

    /// What `cargo run` writes on standard error ahead of the run: the workspace's compiler
    /// warnings, replayed from cache, led by a line that merely looks like a Blocking one.
    fn Cargo_Replay() -> String
    {
        let warnings = (0..LINES_OF_OTHER_OUTPUT).map(|number| {
            return format!(
                "warning: unreachable expression\n  --> crates\\platform\\nomos-platform-std\\src\\launcher\\wait.rs:{number}:5\n"
            );
        });

        return format!("{TOOLCHAIN_LOOKALIKE_LINE}\n{}", warnings.collect::<String>());
    }

    /// What a refusal kept of a step's output before the report was read apart from it: the
    /// last 2,000 bytes of standard output followed by standard error. Spelled out here rather
    /// than borrowed from the code under test, so the code cannot move the expectation with it.
    fn Tail_Before_The_Report_Was_Read_Apart(output: &ProgramOutput) -> String
    {
        return super::super::Tail_Of(&format!("{}{}", output.stdout, output.stderr), super::super::refusal::OUTPUT_TAIL_LIMIT);
    }

    /// A step that exited with `code` having printed `stdout` and `stderr`.
    fn Printed(code: i32, stdout: String, stderr: String) -> ProgramOutput
    {
        return ProgramOutput { outcome: ExitOutcome::Exited { code }, stdout, stderr };
    }

    /// A launcher whose lint step and `Rules` step each answer with their own output.
    struct PerStep
    {
        lint: ProgramOutput,
        rules: ProgramOutput,
    }

    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for PerStep
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl ProgramLauncher for &PerStep
    {
        fn Run(&self, command: &Command) -> Result<ProgramOutput, String>
        {
            let is_lint = command.argv.iter().any(|word| return word == LINT_PROGRAM_WORD);

            return Ok(if is_lint { self.lint.clone() } else { self.rules.clone() });
        }
    }

    /// The refusal [`Run_Gate_Steps`] answers `launcher` with, in a tree of its own named
    /// `name`, and the tail it carries.
    fn Refused_Output_Tail(launcher: &PerStep, name: &str) -> (FinishRefusal, String)
    {
        let directory = Tree_With_Workflow(name, WORKFLOW);
        let clock = FixedClock(NOW_SECONDS);
        let ledger = Ledger_At(&directory, &clock);
        let runner = Runner {
            working_directory: Some(directory.as_path()),
            timeout: std::time::Duration::from_secs(GATE_TIMEOUT_SECONDS),
        };

        let Err(refusal) = Run_Gate_Steps(&ledger, &launcher, &ItemId::New("G-3"), runner)
        else
        {
            panic!("a step that exits nonzero must refuse");
        };
        let FinishRefusal::GateFailed { output_tail, .. } = &refusal
        else
        {
            panic!("expected GateFailed, got {refusal:?}");
        };
        let tail = output_tail.clone();

        return (refusal, tail);
    }

    /// The gap `OD-LEDGER-003` version 2 recorded. A `Rules` step refuses for two Blocking
    /// findings that each sit behind more than 2,000 bytes of the report, with cargo's replay
    /// behind them both, so the tail a refusal kept before holds neither; the refusal its
    /// reader sees still names each by rule, location and summary.
    #[test]
    fn Test_A_Refused_Rules_Step_Should_Show_Every_Blocking_Finding_However_Far_From_The_End_It_Fell()
    {
        let rules = Printed(BLOCKING_EXIT_CODE, Report_With_Blocking_Lines_Far_From_Its_End(), Cargo_Replay());
        let before = Tail_Before_The_Report_Was_Read_Apart(&rules);
        assert!(
            !before.contains(&Blocking_Line()) && !before.contains(SECOND_BLOCKING_LINE),
            "the case is only a case if the old tail reached neither line: {before}"
        );
        let launcher = PerStep { lint: Printed(0, String::new(), String::new()), rules };

        let (refusal, _) = Refused_Output_Tail(&launcher, "rules-lifts-blocking");

        let described = refusal.Describe();
        for shown in [Blocking_Line().as_str(), SECOND_BLOCKING_LINE, BLOCKING_RULE, BLOCKING_LOCATION, BLOCKING_SUMMARY]
        {
            assert!(described.contains(shown), "the refusal must show `{shown}`: {described}");
        }
    }

    /// What is lifted is the report's own Blocking lines and nothing else: not its advisory
    /// findings, and not a line on cargo's stream that merely begins the same way. Everything
    /// the refusal carried before still follows, unchanged, at its end.
    #[test]
    fn Test_A_Refused_Rules_Step_Should_Lift_Only_The_Reports_Blocking_Lines_And_Keep_The_Old_Tail()
    {
        let rules = Printed(BLOCKING_EXIT_CODE, Report_With_Blocking_Lines_Far_From_Its_End(), Cargo_Replay());
        let before = Tail_Before_The_Report_Was_Read_Apart(&rules);
        assert!(!before.contains(TOOLCHAIN_LOOKALIKE_LINE), "the lookalike must sit outside the old tail: {before}");
        let launcher = PerStep { lint: Printed(0, String::new(), String::new()), rules };

        let (_, tail) = Refused_Output_Tail(&launcher, "rules-lifts-only-blocking");

        let Some(lifted) = tail.strip_suffix(before.as_str())
        else
        {
            panic!("the old tail must close the refusal unchanged: {tail}");
        };
        let lifted_count = lifted.lines().filter(|line| return line.starts_with(BLOCKING_LINE_PREFIX)).count();
        assert_eq!(lifted_count, BLOCKING_LINES_IN_THE_REPORT, "{lifted}");
        assert!(!lifted.contains("[Advisory]"), "an advisory finding is not lifted: {lifted}");
        assert!(!tail.contains(TOOLCHAIN_LOOKALIKE_LINE), "cargo's stream is not the report: {tail}");
    }

    /// The lint step's refusal is not made worse. It prints no report line labelled Blocking,
    /// so it refuses with exactly the tail it carried before, byte for byte, whatever it wrote
    /// on either stream.
    #[test]
    fn Test_A_Refused_Lint_Step_Should_Carry_Exactly_The_Tail_It_Carried_Before()
    {
        let errors: String = (0..LINES_OF_OTHER_OUTPUT)
            .map(|number| return format!("error: this argument is passed by value\n  --> crates/a/src/lib.rs:{number}:1\n"))
            .collect();
        let lint = Printed(LINT_EXIT_CODE, "clippy's own standard output\n".to_owned(), errors);
        let before = Tail_Before_The_Report_Was_Read_Apart(&lint);
        let launcher = PerStep { lint, rules: Printed(0, String::new(), String::new()) };

        let (refusal, tail) = Refused_Output_Tail(&launcher, "lint-tail-unchanged");

        assert!(matches!(refusal, FinishRefusal::GateFailed { exit_code: LINT_EXIT_CODE, .. }), "got {refusal:?}");
        assert_eq!(tail, before);
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
