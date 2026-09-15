//! The outcome each verb reduces to, exercised: the `ExitCode` a run reports, and the causes
//! that leave it without a verdict.

use super::super::{ExitCode, Render_Run};
use super::{Empty_Findings, Example_Finding};
use nomos_check_orchestration::{CheckOutcome, Claim, Examined};
use nomos_contracts::{Finding, GateCategory};
use nomos_gate_orchestration::{Fresh_Run_Id, GateFindings, GateRunOutcome, GateRunResult, NoVerdict};
use nomos_platform::Timestamp;
use std::path::PathBuf;

/// A run that judged `finding` and came out with no verdict, for `cause`.
fn Judged_Without_A_Verdict(finding: &Finding, cause: Option<NoVerdict>) -> GateRunResult
{
    return GateRunResult {
        // These tests are about rendering, and say nothing about what judged the run.
        provenance: None,
        no_verdict: cause,
        unmatched_policy: Vec::new(),
        run: Fresh_Run_Id(Timestamp::From_Unix_Seconds(0)),
        root: PathBuf::from("."),
        check_outcome: CheckOutcome::Judged {
            findings: vec![finding.clone()],
            examined: Examined { files: 1, facts: 1 },
            claim: Claim::Incomplete,
        },
        findings: Empty_Findings(),
        disposition: GateRunOutcome::Indeterminate,
    };
}

/// A judged run that judged `findings`, found `whole`, and reached `disposition`.
///
/// Root, run id, provenance and the examined counts are held fixed here, so a test that uses
/// this differs from another in the finding it judged and in nothing else -- which is what
/// makes each one's exit code attributable to that difference.
fn Judged_Run(findings: Vec<Finding>, whole: GateFindings, disposition: GateRunOutcome) -> GateRunResult
{
    return GateRunResult {
        // These tests are about rendering, and say nothing about what judged the run.
        provenance: None,
        no_verdict: None,
        unmatched_policy: Vec::new(),
        run: Fresh_Run_Id(Timestamp::From_Unix_Seconds(0)),
        root: PathBuf::from("."),
        check_outcome: CheckOutcome::Judged { findings, examined: Examined { files: 1, facts: 1 }, claim: Claim::Complete },
        findings: whole,
        disposition,
    };
}

/// A check outcome that never reached `Judged` must render as `Vacuous`, the same claim
/// `Render_Check_Unreadable`'s siblings already make for `run`'s own non-judged arms --
/// `Render_Run` picks the same arm for `explain`'s `NoSource`.
#[test]
fn Test_Render_Run_Should_Report_Vacuous_When_The_Check_Outcome_Never_Reached_Judged()
{
    let result = GateRunResult {
        provenance: None,
        no_verdict: None,
        unmatched_policy: Vec::new(),
        run: Fresh_Run_Id(Timestamp::From_Unix_Seconds(0)),
        root: PathBuf::from("does/not/matter"),
        check_outcome: CheckOutcome::NoSource,
        findings: Empty_Findings(),
        disposition: GateRunOutcome::Indeterminate,
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Run(&result, &mut stdout, &mut stderr);

    let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
    assert_eq!(code, ExitCode::Vacuous, "{rendered_stderr}");
    assert!(rendered_stderr.contains("nothing was judged"), "{rendered_stderr}");
    assert!(String::from_utf8_lossy(&stdout).is_empty());
}

/// A judged run whose disposition is `Indeterminate` says there is no verdict, says which
/// mechanism produced that, and exits `Contradictory` -- instead of aborting the process.
///
/// The arm these cover was `unreachable!` until it was measured, on the claim that
/// `Run_Gate` only assigns `Indeterminate` to a run which never reached `Judged`. It
/// assigns it to a judged run in three cases, and a real `nomos gate run` aborted with
/// 101 on every one.
///
/// One test per cause rather than one over all of them, because the whole point of
/// carrying a cause is that the three read differently to a person: two send a reader to
/// the policy file, and the third tells them not to go looking for a fault at all.
#[test]
fn Test_Render_Run_Should_Name_A_Malformed_Policy_As_The_Reason_There_Is_No_Verdict()
{
    let finding = Example_Finding(GateCategory::Advisory);
    let result = Judged_Without_A_Verdict(&finding, Some(NoVerdict::MalformedPolicy("unknown field `basline`".to_owned())));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Run(&result, &mut stdout, &mut stderr);

    let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
    assert_eq!(code, ExitCode::Contradictory, "{rendered_stderr}");
    assert!(rendered_stderr.contains("not a policy this reader accepts"), "{rendered_stderr}");
    assert!(rendered_stderr.contains("basline"), "{rendered_stderr}");
    // The abort came *after* the findings were written, so what it destroyed was the
    // verdict line and the exit code. A fix that reported the state by dropping the
    // report would be the worse answer.
    assert!(String::from_utf8_lossy(&stdout).contains(&finding.Describe()));
}

#[test]
fn Test_Render_Run_Should_Name_An_Unreadable_Policy_Separately_From_A_Malformed_One()
{
    let finding = Example_Finding(GateCategory::Advisory);
    let result = Judged_Without_A_Verdict(&finding, Some(NoVerdict::UnreadablePolicy("nomos-gate.json: PermissionDenied".to_owned())));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Run(&result, &mut stdout, &mut stderr);

    let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
    assert_eq!(code, ExitCode::Contradictory, "{rendered_stderr}");
    assert!(rendered_stderr.contains("could not be read"), "{rendered_stderr}");
    assert!(rendered_stderr.contains("PermissionDenied"), "{rendered_stderr}");
}

/// The coverage floor is the one cause where nothing is wrong, so its rendering says so
/// rather than sending a reader to look for a fault that is not there.
#[test]
fn Test_Render_Run_Should_Say_Nothing_Is_Wrong_When_The_Coverage_Floor_Withheld_The_Pass()
{
    let finding = Example_Finding(GateCategory::Advisory);
    let result = Judged_Without_A_Verdict(&finding, Some(NoVerdict::IncompleteCoverage));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Run(&result, &mut stdout, &mut stderr);

    let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
    assert_eq!(code, ExitCode::Contradictory, "{rendered_stderr}");
    assert!(rendered_stderr.contains("require-completeness"), "{rendered_stderr}");
    assert!(rendered_stderr.contains("Nothing is wrong"), "{rendered_stderr}");
}

/// A cause `Run_Gate` does not currently leave unset is still answered rather than
/// asserted away: an unreachable claim about this exact arm is what aborted the process
/// before it carried one, and the honest rendering of a missing reason says it is missing.
#[test]
fn Test_Render_Run_Should_Say_The_Reason_Is_Missing_Rather_Than_Assume_One()
{
    let finding = Example_Finding(GateCategory::Advisory);
    let result = Judged_Without_A_Verdict(&finding, None);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Run(&result, &mut stdout, &mut stderr);

    let rendered_stderr = String::from_utf8_lossy(&stderr).into_owned();
    assert_eq!(code, ExitCode::Contradictory, "{rendered_stderr}");
    assert!(rendered_stderr.contains("did not record why"), "{rendered_stderr}");
}

/// A judged run with nothing blocking reports `Ok` and names its own `RunId` -- the
/// "real work" arm `Report_Judged` does, end to end at this function's own boundary.
#[test]
fn Test_Render_Run_Should_Report_The_RunId_And_Ok_When_Nothing_Blocks()
{
    let result = Judged_Run(Vec::new(), Empty_Findings(), GateRunOutcome::Passed);
    // Read back from the result rather than built beside it: `Fresh_Run_Id` counts the calls
    // made in this process, so a second one handed the same timestamp would be a different
    // identity, and the assertion below would then fail for a reason that says nothing about
    // the rendering. What is asserted is that the run's own identity is the one printed.
    let run = result.run;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Run(&result, &mut stdout, &mut stderr);

    let rendered = String::from_utf8_lossy(&stdout).into_owned();
    assert_eq!(code, ExitCode::Ok, "{rendered}");
    assert!(rendered.contains(&format!("run: {run}")), "{rendered}");
    assert!(rendered.contains("0 finding(s), 0 of which can fail a build"), "{rendered}");
    assert!(String::from_utf8_lossy(&stderr).is_empty());
}

/// A judged run with one blocking finding reports `Violations` and names the finding --
/// `Exit_Code_For`'s `Failed` arm, only reachable through `Report_Judged`.
#[test]
fn Test_Render_Run_Should_Report_Violations_When_A_Finding_Blocks()
{
    let finding = Example_Finding(GateCategory::Blocking);
    let whole = GateFindings { blocking_findings: vec![finding.clone()], ..Empty_Findings() };
    let result = Judged_Run(vec![finding.clone()], whole, GateRunOutcome::Failed);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = Render_Run(&result, &mut stdout, &mut stderr);

    let rendered = String::from_utf8_lossy(&stdout).into_owned();
    assert_eq!(code, ExitCode::Violations, "{rendered}");
    assert!(rendered.contains(&finding.Describe()), "{rendered}");
    assert!(rendered.contains("1 finding(s), 1 of which can fail a build"), "{rendered}");
}
