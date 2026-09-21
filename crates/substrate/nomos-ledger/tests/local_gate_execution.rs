//! A local execution of the canonical gate step set, against the real workflow and a
//! scripted launcher.
//!
//! `OD-GATE-033` decided that the model is `.github/workflows/gate.yml` itself, that a local
//! execution gives every step one of three values, and that a run reports the set it did not
//! execute. These assert that against the file this repository actually ships, so a workflow
//! edit that moved a guard would show up here rather than in a wrong local verdict.

use nomos_ledger::{Derive_Step, Derive_Steps, GateUnknown, LINT_STEP, Run_Gate_Locally, StepExecution, StepGuard};
use nomos_platform::{
    Command, DeterminismStrength, ExitOutcome, ProgramLauncher, ProgramOutput, ReproducibilityScope, Strategy,
    TraceEquivalence,
};

const WORKFLOW: &str = include_str!("../../../../.github/workflows/gate.yml");

const LINUX: &str = "ubuntu-latest";
const WINDOWS: &str = "windows-latest";

/// The fifteen steps the workflow declares, and the split `OD-GATE-033` enumerated.
const DECLARED_STEPS: usize = 15;
const UNGUARDED_STEPS: usize = 4;
const WINDOWS_ONLY_STEPS: usize = 1;
/// How many of the fifteen a Linux host really runs: the rest are one Windows-only step and
/// two whose `run:` is a shell script the argv subset refuses.
const LOCALLY_EXECUTED_ON_LINUX: usize = 12;

/// A launcher that runs nothing and reports a fixed exit code, so a whole-set execution is
/// driven without starting a single process.
struct Scripted(i32);

impl Strategy for Scripted
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for Scripted
{
    fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
    {
        return Ok(ProgramOutput {
            outcome: ExitOutcome::Exited { code: self.0 },
            stdout: String::new(),
            stderr: String::new(),
        });
    }
}

/// A launcher that could not start the process at all.
struct NeverStarts;

impl Strategy for NeverStarts
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for NeverStarts
{
    fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
    {
        return Err("no such program".to_owned());
    }
}

#[test]
fn Test_The_Reader_Should_Derive_Every_Step_The_Workflow_Declares()
{
    let steps = Derive_Steps(WORKFLOW);

    assert_eq!(steps.len(), DECLARED_STEPS, "{:?}", steps.iter().map(|step| return &step.name).collect::<Vec<_>>());
}

/// The split `OD-GATE-033`'s own table states, re-derived from the file rather than retyped.
#[test]
fn Test_The_Reader_Should_Split_The_Steps_The_Way_The_Record_Enumerated_Them()
{
    let steps = Derive_Steps(WORKFLOW);

    let unguarded = steps.iter().filter(|step| return step.guard == StepGuard::Unguarded).count();
    let windows = steps.iter().filter(|step| return step.guard == StepGuard::Host(WINDOWS.to_owned())).count();
    let linux = steps.iter().filter(|step| return step.guard == StepGuard::Host(LINUX.to_owned())).count();

    assert_eq!(unguarded, UNGUARDED_STEPS);
    assert_eq!(windows, WINDOWS_ONLY_STEPS);
    assert_eq!(linux, DECLARED_STEPS - UNGUARDED_STEPS - WINDOWS_ONLY_STEPS);
}

/// The single-step derivation `work finish` depends on keeps answering exactly as it did.
#[test]
fn Test_The_Step_Set_Should_Not_Disturb_The_Single_Step_Derivation()
{
    let derived = Derive_Step(WORKFLOW, LINT_STEP).expect("the workflow has a Lint step");
    let steps = Derive_Steps(WORKFLOW);
    let lint = steps.iter().find(|step| return step.name == LINT_STEP).expect("the set has it too");

    assert_eq!(lint.argv.as_ref().expect("the Lint step derives"), &derived);
}

/// The falsifier for the guard reading. Remove it and `Determinism` stops being reported as
/// unavailable on a Linux host, which is the exact defect `OD-GATE-033` names.
#[test]
fn Test_A_Windows_Only_Step_Should_Be_Unavailable_On_A_Linux_Host()
{
    let steps = Derive_Steps(WORKFLOW);
    let run = Run_Gate_Locally(&steps, LINUX, &Scripted(0), None);

    let determinism = run
        .Steps()
        .iter()
        .find(|(name, _)| return name == "Determinism")
        .expect("the workflow declares a Determinism step");

    assert_eq!(determinism.1, StepExecution::Unavailable { host: LINUX.to_owned() });
}

/// The same step on the host its guard names really runs.
#[test]
fn Test_A_Windows_Only_Step_Should_Execute_On_A_Windows_Host()
{
    let steps = Derive_Steps(WORKFLOW);
    let run = Run_Gate_Locally(&steps, WINDOWS, &Scripted(0), None);

    let determinism = run
        .Steps()
        .iter()
        .find(|(name, _)| return name == "Determinism")
        .expect("the workflow declares a Determinism step");

    assert_eq!(determinism.1, StepExecution::Executed { code: 0 });
}

/// The falsifier for the unexecuted-set report, and the measurement of how complete a local
/// gate run can be today.
///
/// Three of the fifteen steps do not run on a Linux host, for two different reasons, and the
/// report has to keep them apart rather than merge them into one count. `Determinism` is
/// guarded to Windows. `Show toolchain` and `Portability floor` are admitted here and still
/// do not run, because their `run:` bodies are shell scripts -- one joins two commands with
/// `&&`, the other is a block -- and the argv subset refuses a script rather than guessing
/// at it. So a local run on Linux executes twelve steps, and the three it does not execute
/// are named.
///
/// This test reads the report rather than the per-step values, so removing the report fails
/// it while every value stays right.
#[test]
fn Test_A_Local_Run_Should_Name_The_Steps_It_Did_Not_Execute()
{
    let steps = Derive_Steps(WORKFLOW);
    let run = Run_Gate_Locally(&steps, LINUX, &Scripted(0), None);

    let unexecuted = run.Unexecuted();

    assert_eq!(unexecuted, vec!["Show toolchain", "Determinism", "Portability floor"], "{unexecuted:?}");
    assert_eq!(run.Steps().len() - unexecuted.len(), LOCALLY_EXECUTED_ON_LINUX);
}

/// The two reasons a step goes unexecuted stay apart in the values, which is the whole
/// content of `OD-GATE-033`'s decision 2: a step no host could run here is not the same as a
/// step this reader could not understand, and neither is a pass.
#[test]
fn Test_The_Unexecuted_Steps_Should_Keep_Their_Two_Reasons_Apart()
{
    let steps = Derive_Steps(WORKFLOW);
    let run = Run_Gate_Locally(&steps, LINUX, &Scripted(0), None);

    let value_of = |wanted: &str| {
        return run
            .Steps()
            .iter()
            .find(|(name, _)| return name == wanted)
            .map(|(_, execution)| return execution.clone())
            .expect("the workflow declares it");
    };

    assert_eq!(value_of("Determinism"), StepExecution::Unavailable { host: LINUX.to_owned() });
    assert!(
        matches!(value_of("Show toolchain"), StepExecution::Refused(GateUnknown::NotASingleCommand { .. })),
        "a shell script is refused, not reported unavailable"
    );
    assert!(matches!(value_of("Portability floor"), StepExecution::Refused(GateUnknown::NotASingleCommand { .. })));
}

/// The falsifier for the guard refusal. A guard outside the subset must refuse the step
/// rather than silently decide which leg it belongs to.
#[test]
fn Test_A_Guard_Outside_The_Subset_Should_Refuse_The_Step()
{
    let injected = WORKFLOW.replace(
        "if: matrix.os == 'windows-latest'",
        "if: always() && matrix.os != 'ubuntu-latest'",
    );
    assert_ne!(injected, WORKFLOW, "the injection must actually change the workflow");

    let steps = Derive_Steps(injected.as_str());
    let run = Run_Gate_Locally(&steps, LINUX, &Scripted(0), None);

    let determinism = run
        .Steps()
        .iter()
        .find(|(name, _)| return name == "Determinism")
        .expect("the workflow declares a Determinism step");

    let StepExecution::Refused(GateUnknown::GuardOutsideSubset { step, .. }) = &determinism.1
    else
    {
        panic!("a guard outside the subset must refuse, got {:?}", determinism.1);
    };
    assert_eq!(step, "Determinism");
}

/// A step that was admitted and never exited is not an exit code, and is not a pass.
#[test]
fn Test_A_Step_That_Never_Started_Should_Refuse_Rather_Than_Report_An_Exit()
{
    let steps = Derive_Steps(WORKFLOW);
    let run = Run_Gate_Locally(&steps, LINUX, &NeverStarts, None);

    let lint = run
        .Steps()
        .iter()
        .find(|(name, _)| return name == LINT_STEP)
        .expect("the workflow declares a Lint step");

    assert!(matches!(lint.1, StepExecution::Refused(GateUnknown::DidNotRun { .. })), "{:?}", lint.1);
}

/// A failing step is reported with the code it failed with, not as a refusal.
#[test]
fn Test_A_Failing_Step_Should_Carry_Its_Own_Exit_Code()
{
    const FAILED: i32 = 101;

    let steps = Derive_Steps(WORKFLOW);
    let run = Run_Gate_Locally(&steps, LINUX, &Scripted(FAILED), None);

    let lint = run
        .Steps()
        .iter()
        .find(|(name, _)| return name == LINT_STEP)
        .expect("the workflow declares a Lint step");

    assert_eq!(lint.1, StepExecution::Executed { code: FAILED });
}
