//! The `explain` verb's rendering, exercised.

use super::super::{ExitCode, Render_Explain};
use super::Example_Finding;
use nomos_check_orchestration::{CheckOutcome, Claim, Examined, SupportingFactTrail};
use nomos_contracts::{EvidenceClass, Finding, GateCategory};
use nomos_gate_orchestration::{Explanation, GateExplainResult};
use std::path::PathBuf;

/// A judged tree in which `findings` are the only findings, explained by `explanation`.
///
/// The root, the examined counts and the claim are held fixed here, so the tests that use
/// this differ in what was found and in nothing else.
fn Judged_Explain(findings: Vec<Finding>, explanation: Explanation) -> GateExplainResult
{
    return GateExplainResult {
        root: PathBuf::from("."),
        check_outcome: CheckOutcome::Judged { findings, examined: Examined { files: 1, facts: 1 }, claim: Claim::Complete, supporting_facts: SupportingFactTrail::New() },
        explanation,
    };
}

/// `finding`, as `explain` reports it when the run that met it would have blocked.
///
/// Every other attribution is `None` because the subject here is the block status and its
/// wording; a test about what calibrated or suppressed a finding passes one of those instead.
fn Explained_As_Found(finding: &Finding) -> Explanation
{
    return Explanation::Found {
        finding: Box::new(finding.clone()),
        would_block: true,
        floored_by: None,
        calibrated_by: None,
        suppressed_by: None,
        baselined_by: None,
        contract: None,
    };
}

/// `explain` shares `run`'s own non-judged rendering, so a check outcome that never
/// reached `Judged` must report `Vacuous` here too, regardless of what `explanation`
/// carries.
#[test]
fn Test_Render_Explain_Should_Report_Vacuous_When_The_Check_Outcome_Never_Reached_Judged()
{
    let result = GateExplainResult {
        root: PathBuf::from("does/not/matter"),
        check_outcome: CheckOutcome::NoSource,
        explanation: Explanation::NotFound,
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Explain(&result, &mut stdout, &mut stderr);

    let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
    assert_eq!(code, ExitCode::Vacuous, "{rendered_stderr}");
    assert!(rendered_stderr.contains("the query cannot be answered"), "{rendered_stderr}");
}

/// A judged tree in which no finding names the query's location answers `not found` and
/// exits clean.
#[test]
fn Test_Render_Explain_Should_Report_Not_Found_When_Judged_And_No_Finding_Matches()
{
    let result = Judged_Explain(Vec::new(), Explanation::NotFound);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Explain(&result, &mut stdout, &mut stderr);

    let rendered = String::from_utf8_lossy(&stdout).into_owned();
    assert_eq!(code, ExitCode::Ok, "{rendered}");
    assert!(rendered.contains("not found"), "{rendered}");
    assert!(String::from_utf8_lossy(&stderr).is_empty());
}

/// A found finding that would block a real run reports `Violations` and names both the
/// finding and its block status -- `Report_Found`'s own "real work" arm.
#[test]
fn Test_Render_Explain_Should_Report_Would_Block_For_A_Found_Blocking_Finding()
{
    let finding = Example_Finding(GateCategory::Blocking);
    let result = Judged_Explain(vec![finding.clone()], Explained_As_Found(&finding));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Explain(&result, &mut stdout, &mut stderr);

    let rendered = String::from_utf8_lossy(&stdout).into_owned();
    assert_eq!(code, ExitCode::Violations, "{rendered}");
    assert!(rendered.contains("would block: true"), "{rendered}");
    assert!(rendered.contains(&finding.Describe()), "{rendered}");
}

/// A finding this gate's evidence floor kept from blocking cites the floor, under a label of
/// its own.
///
/// `OD-GATE-034`: a reader told "calibrated" about a finding whose evidence was simply too
/// weak under this gate would go looking for a calibration nobody wrote, so the floor's line
/// names the class it requires and borrows none of the other three labels.
#[test]
fn Test_Render_Explain_Should_Cite_The_Evidence_Floor_That_Moved_A_Finding()
{
    let finding = Example_Finding(GateCategory::Blocking);
    let floored = Explanation::Found {
        finding: Box::new(finding.clone()),
        would_block: false,
        floored_by: Some(EvidenceClass::Derived),
        calibrated_by: None,
        suppressed_by: None,
        baselined_by: None,
        contract: None,
    };
    let result = Judged_Explain(vec![finding], floored);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Explain(&result, &mut stdout, &mut stderr);

    let rendered = String::from_utf8_lossy(&stdout).into_owned();
    assert_eq!(code, ExitCode::Ok, "a finding under the floor does not fail the build: {rendered}");
    assert!(rendered.contains("would block: false"), "{rendered}");
    assert!(
        rendered.contains("below the evidence floor: this gate requires at least Derived"),
        "the floor that moved it must be citable: {rendered}"
    );
    assert!(!rendered.contains("calibrated by"), "and must not borrow another label: {rendered}");
}
