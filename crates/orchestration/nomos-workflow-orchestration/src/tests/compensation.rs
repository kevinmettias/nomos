//! Honoring `Compensation`: what a failed run undoes, what it refuses to undo, and what
//! it reports as owed to whoever assembled the plan.

use super::support::*;
use super::*;

/// The gate fixture that fails: five value parameters against the one rule
/// [`super::Gate_Body_Over`] narrows to.
const OVER_THE_PARAMETER_LIMIT: &str = "pub fn Something(a: i32, b: i32, c: i32, d: i32, e: i32) {}\n";

/// The one file a correction step commits, and the one a compensation puts back.
const CORRECTED_PATH: &str = "a.rs";

/// The two halves of the reverse-order requirement in one plan: a correction step whose
/// own body can undo its write, an externally compensated step after it, and a failing
/// gate step to fail the run. Compensation runs from the last completed step backward, so
/// the externally compensated step is reported before the correction is put back.
#[test]
fn Test_A_Failed_Run_Should_Compensate_Its_Completed_Steps_In_Reverse_Order()
{
    let root = Fresh_Root("nomos-workflow-orchestration-compensation-reverse-order");
    let path = root.join(CORRECTED_PATH);
    std::fs::write(&path, PHANTOM_FIXTURE).expect("the fixture root Fresh_Root just created holds this file");
    let plan = Unwinding_Plan(&root, Compensation::ExternallyCompensated);

    let run = Ran_Report(&plan, vec![Clean_Claude_Code_Response("second")]);
    let after_compensation = std::fs::read_to_string(&path).expect("the compensated correction left the file readable");
    let _ignored = std::fs::remove_dir_all(&root);

    assert!(matches!(run.outcome, WorkflowOutcome::Failed { index: 2, .. }), "the gate step failed the run: {:?}", run.outcome);
    assert_eq!(
        run.compensations,
        vec![StepCompensation::Owed { index: 1 }, StepCompensation::Compensated { index: 0, restored: vec![CORRECTED_PATH.to_owned()] }],
        "the later step is reported first, and the correction is put back second"
    );
    assert_eq!(after_compensation, PHANTOM_FIXTURE, "the committed correction was undone byte for byte");
}

/// A body with no compensating mode that declared `SelfCompensating` is reported as a
/// refused declaration rather than skipped: an agent step re-invoked in a compensating
/// mode is the same dispatch again, not the reverse of it.
#[test]
fn Test_A_Self_Compensating_Body_With_No_Compensating_Mode_Should_Be_Reported_As_Refused()
{
    let plan = [
        WorkflowStepPlan {
            declaration: Declaring_Compensation(Compensation::SelfCompensating),
            body: Agent_Step(nomos_agent_orchestration::Backend::ClaudeCode, Task_Envelope("first")),
        },
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Gate(Gate_Body_Over(OVER_THE_PARAMETER_LIMIT)) },
    ];

    let run = Ran_Report(&plan, vec![Clean_Claude_Code_Response("first")]);

    assert!(matches!(run.outcome, WorkflowOutcome::Failed { index: 1, .. }), "{:?}", run.outcome);
    let refusal = run.compensations.first().expect("the completed agent step declared a compensation");
    assert!(matches!(refusal, StepCompensation::Refused { index: 0, .. }), "{refusal:?}");
    assert_eq!(run.compensations.len(), 1, "the failing gate step itself declared no compensation");
}

/// A step declaring `Compensation::None` asked for nothing, and is not reported at all --
/// the three answers that matter are not buried under one entry per step in the plan.
#[test]
fn Test_A_Step_Declaring_No_Compensation_Should_Not_Be_Reported_At_All()
{
    let plan = [
        WorkflowStepPlan { declaration: Coherent_Step(), body: Agent_Step(nomos_agent_orchestration::Backend::ClaudeCode, Task_Envelope("first")) },
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Gate(Gate_Body_Over(OVER_THE_PARAMETER_LIMIT)) },
    ];

    let run = Ran_Report(&plan, vec![Clean_Claude_Code_Response("first")]);

    assert!(matches!(run.outcome, WorkflowOutcome::Failed { index: 1, .. }), "{:?}", run.outcome);
    assert!(run.compensations.is_empty(), "nothing declared a compensation, so nothing is reported: {:?}", run.compensations);
}

/// Compensation answers to a failure, not to every run: the identical correction step in a
/// plan that completes is left committed, and nothing is reported as compensated.
#[test]
fn Test_A_Completed_Run_Should_Compensate_Nothing()
{
    let root = Fresh_Root("nomos-workflow-orchestration-compensation-completed-run");
    let path = root.join(CORRECTED_PATH);
    std::fs::write(&path, PHANTOM_FIXTURE).expect("the fixture root Fresh_Root just created holds this file");
    let plan = [WorkflowStepPlan {
        declaration: Declaring_Compensation(Compensation::SelfCompensating),
        body: Body::Correction(Correction_Body_Over(&root, PHANTOM_FIXTURE)),
    }];

    let run = Ran_Report(&plan, Vec::new());
    let after_run = std::fs::read_to_string(&path).expect("the committed correction left the file readable");
    let _ignored = std::fs::remove_dir_all(&root);

    assert!(matches!(run.outcome, WorkflowOutcome::Completed { .. }), "{:?}", run.outcome);
    assert!(run.compensations.is_empty(), "a run that completed has nothing to undo: {:?}", run.compensations);
    assert_ne!(after_run, PHANTOM_FIXTURE, "the correction stayed committed");
}

/// A correction step that staged without committing wrote no file, so its own compensating
/// mode ran and put nothing back. That is reported as a compensation restoring nothing,
/// not as a refused declaration: the mode the body supports ran.
#[test]
fn Test_A_Staged_Correction_Step_Should_Report_A_Compensation_That_Restored_Nothing()
{
    let root = Fresh_Root("nomos-workflow-orchestration-compensation-staged-only");
    let path = root.join(CORRECTED_PATH);
    std::fs::write(&path, PHANTOM_FIXTURE).expect("the fixture root Fresh_Root just created holds this file");
    let staged = CorrectionBody::New(root.clone(), Phantom_Source(), CommitIntent::Stage);
    let plan = [
        WorkflowStepPlan { declaration: Declaring_Compensation(Compensation::SelfCompensating), body: Body::Correction(staged) },
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Gate(Gate_Body_Over(OVER_THE_PARAMETER_LIMIT)) },
    ];

    let run = Ran_Report(&plan, Vec::new());
    let untouched = std::fs::read_to_string(&path).expect("a staged correction left the file readable");
    let _ignored = std::fs::remove_dir_all(&root);

    assert_eq!(run.compensations, vec![StepCompensation::Compensated { index: 0, restored: Vec::new() }]);
    assert_eq!(untouched, PHANTOM_FIXTURE, "nothing was written, so nothing was put back");
}

/// The three-step plan the reverse-order case runs: a committing correction over `root`
/// declaring `SelfCompensating`, an agent step declaring `middle`, and a gate step that
/// fails.
fn Unwinding_Plan(root: &std::path::Path, middle: Compensation) -> Vec<WorkflowStepPlan>
{
    return vec![
        WorkflowStepPlan {
            declaration: Declaring_Compensation(Compensation::SelfCompensating),
            body: Body::Correction(Correction_Body_Over(root, PHANTOM_FIXTURE)),
        },
        WorkflowStepPlan {
            declaration: Declaring_Compensation(middle),
            body: Agent_Step(nomos_agent_orchestration::Backend::ClaudeCode, Task_Envelope("second")),
        },
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Gate(Gate_Body_Over(OVER_THE_PARAMETER_LIMIT)) },
    ];
}

/// The already-walked source a correction body carries: the phantom-mirror fixture, filed
/// under the one path a correction run reports having written.
fn Phantom_Source() -> Vec<nomos_rules::SourceFile>
{
    return vec![nomos_rules::SourceFile::New(CORRECTED_PATH, nomos_model::Subject_Of_Path(CORRECTED_PATH), PHANTOM_FIXTURE.to_owned())];
}
