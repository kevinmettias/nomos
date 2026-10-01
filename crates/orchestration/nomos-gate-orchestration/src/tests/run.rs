//! What a whole run does with a real judged tree: the three policies that keep a finding from
//! blocking, and the two selectors that decide what is judged and what counts.

use super::{
    Assert_Tolerated_Not_Blocking, Baseline_Of, Calibration_Of, Clean_Source, Command_At, Command_With_Baseline,
    Command_With_Calibration, Command_With_Suppression, Mirrored_Source, One_Real_Blocking_Finding, Ran_Over, Ran_Unwalked,
    Repository_Root, SourcePath, Suppression_Of,
};
use crate::{AdoptionPolicy, BaselinePolicy, GateCommand, GateRunOutcome, RuleSelector, ScopeSelector, SuppressionPolicy};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{Finding, RuleId};
use nomos_rules::{COMPLETENESS_MIRROR, NAMING_CONVENTION};

/// The findings a judged `check_outcome` carries, or a panic naming what every fixture that
/// reaches this helper has already asserted -- `Judged`, checked once here rather than
/// re-destructured at each call site.
fn Judged_Findings(check_outcome: &CheckOutcome) -> &[Finding]
{
    let CheckOutcome::Judged { findings, .. } = check_outcome
    else
    {
        panic!("expected a judged check outcome");
    };

    return findings;
}

/// A root that was never walked -- the composition root's own `None`, the same case
/// `crates/host/nomos-cli/src/gate/run.rs` currently assigns `CheckOutcome::Unreadable` for
/// by hand. [`crate::Run_Gate`] must make the identical assignment, since this crate now
/// performs that composition too.
#[test]
fn Test_Judged_Sources_Should_Report_Unreadable_For_An_Unwalked_Root()
{
    let result = Ran_Unwalked(&Command_At(Repository_Root()));

    assert!(matches!(result.check_outcome, CheckOutcome::Unreadable));
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(result.findings.blocking_findings.is_empty());
}

/// A directory that was walked and held nothing -- `Some(Vec::new())` -- is a different
/// claim than a root nobody could walk at all, and must not collapse into the same
/// variant.
#[test]
fn Test_A_Walk_That_Found_No_Source_Should_Be_Indeterminate()
{
    let result = Ran_Over(Vec::new(), &Command_At(Repository_Root()));

    assert!(matches!(result.check_outcome, CheckOutcome::NoSource));
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(result.findings.blocking_findings.is_empty());
}

/// A clean source, judged through the real `nomos_check_orchestration::Run` this crate now
/// calls directly, must pass -- the same fixture `nomos-check-orchestration`'s own
/// `Test_A_Clean_Tree_Should_Be_Judged_Complete_With_No_Findings` already proves clean
/// under all four shipped rules.
#[test]
fn Test_A_Clean_Source_Should_Pass()
{
    let sources = vec![Clean_Source(SourcePath("a.rs"))];

    let result = Ran_Over(sources, &Command_At(Repository_Root()));

    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    assert!(result.findings.blocking_findings.is_empty(), "{:?}", result.findings.blocking_findings);
}

/// A phantom mirror -- the same fixture `nomos-check-orchestration`'s own
/// `Test_A_Blocking_Finding_Should_Still_Be_Judged_Complete` uses -- must reach [`crate::Run_Gate`]
/// as a real blocking finding and flip the disposition, proving the reduction this crate now
/// owns runs over `nomos_check_orchestration::Run`'s real output rather than a fixture typed
/// to look like it.
#[test]
fn Test_Run_Gate_Should_Fail_On_A_Blocking_Finding()
{
    let sources = vec![Mirrored_Source(SourcePath("a.rs"))];

    let result = Ran_Over(sources, &Command_At(Repository_Root()));

    assert_eq!(result.disposition, GateRunOutcome::Failed);
    assert!(!result.findings.blocking_findings.is_empty());
    assert!(result
        .findings
        .blocking_findings
        .iter()
        .all(|finding| return finding.gate == nomos_contracts::GateCategory::Blocking));
}

/// [`ScopeSelector`] excludes the one source a walk found, so the run reports
/// `CheckOutcome::NoSource` -- the same answer an empty walk gives, because "scoped to
/// nothing" and "found nothing" still mean the same thing to a caller.
///
/// Unchanged by `OD-GATE-025`, and worth saying why, because that record moved the scope off
/// the source set everywhere else. The judging now happens over the whole walk and is
/// discarded here rather than never running; what a caller is told is identical, which is
/// the point. A mistyped `--include` must not read as a clean pass.
#[test]
fn Test_A_Scoped_Out_Source_Should_Not_Be_Judged()
{
    let sources = vec![Mirrored_Source(SourcePath("a.rs"))];
    let command = GateCommand {
        scope: ScopeSelector { include: vec!["b.rs".to_owned()], exclude: Vec::new() },
        ..Command_At(Repository_Root())
    };

    let result = Ran_Over(sources, &command);

    assert!(matches!(result.check_outcome, CheckOutcome::NoSource));
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(result.findings.blocking_findings.is_empty());
}

/// [`RuleSelector`] excludes the rule behind the one blocking finding this fixture would
/// otherwise produce: per `OD-GATE-017`, `Run` itself now skips a deselected rule's own
/// computation, so the finding never exists at all -- real selection of what runs, not only
/// of what a disposition later discards.
#[test]
fn Test_A_Deselected_Rules_Finding_Should_Not_Exist()
{
    let sources = vec![Mirrored_Source(SourcePath("a.rs"))];
    let command = GateCommand {
        rules: RuleSelector { include: vec![RuleId::New(NAMING_CONVENTION)] },
        ..Command_At(Repository_Root())
    };

    let result = Ran_Over(sources, &command);

    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    assert!(result.findings.blocking_findings.is_empty());
    assert!(
        Judged_Findings(&result.check_outcome)
            .iter()
            .all(|finding| return finding.rule != RuleId::New(COMPLETENESS_MIRROR)),
        "the deselected rule was never asked to run, so its finding must not exist at all"
    );
}

/// A [`crate::Suppression`] matching the one blocking finding this fixture produces: the run
/// still judges the source and `check_outcome` still carries the finding in full, and it now
/// also appears in `suppressed_findings` rather than `blocking_findings` -- suppressed, not
/// silenced.
///
/// The suppression's `subject` is read off a real, unsuppressed run first, rather than
/// recomputed from the file's own path: `Check_Completeness_Mirrors` addresses a finding by
/// the mirrored item's own subject, not the file's -- `Subject_Of_Path` alone does not name
/// it, and this test does not need to know that addressing scheme to prove suppression
/// works over whatever subject a real finding actually carries.
#[test]
fn Test_A_Suppressed_Finding_Should_Not_Block()
{
    let source = || return Mirrored_Source(SourcePath("a.rs"));
    let real_finding = One_Real_Blocking_Finding(source);
    let command = Command_With_Suppression(Repository_Root(), Suppression_Of(&real_finding));

    let result = Ran_Over(vec![source()], &command);

    Assert_Tolerated_Not_Blocking(&result, &result.findings.suppressed_findings);
    assert!(
        Judged_Findings(&result.check_outcome)
            .iter()
            .any(|finding| return finding.rule == RuleId::New(COMPLETENESS_MIRROR)),
        "the suppressed finding must still be judged and carried in check_outcome"
    );
}

/// A [`crate::BaselineDebt`] matching the one blocking finding this fixture produces: the run
/// still judges the source and `check_outcome` still carries the finding in full, and it now
/// also appears in `baselined_findings` rather than `blocking_findings` -- tolerated, not
/// silenced. Mirrors [`Test_A_Suppressed_Finding_Should_Not_Block`] for the second of
/// `Run_Gate`'s two policies.
#[test]
fn Test_A_Baselined_Finding_Should_Not_Block()
{
    let source = || return Mirrored_Source(SourcePath("a.rs"));
    let real_finding = One_Real_Blocking_Finding(source);
    let command = Command_With_Baseline(Repository_Root(), Baseline_Of(&real_finding));

    let result = Ran_Over(vec![source()], &command);

    Assert_Tolerated_Not_Blocking(&result, &result.findings.baselined_findings);
}

/// A finding matched by both a [`crate::Suppression`] and a [`crate::BaselineDebt`] reports as
/// suppressed, not baselined -- `Run_Gate` checks suppression first, so the two lists never
/// double-count the same finding, and this is the one case a passing "not blocking" assertion
/// alone would not catch: `blocking_findings` empty is also true if the ordering were reversed.
#[test]
fn Test_A_Suppressed_And_Baselined_Finding_Should_Report_As_Suppressed()
{
    let source = || return Mirrored_Source(SourcePath("a.rs"));
    let real_finding = One_Real_Blocking_Finding(source);
    let command = GateCommand {
        suppressions: SuppressionPolicy { suppressions: vec![Suppression_Of(&real_finding)] },
        baseline: BaselinePolicy { debt: vec![Baseline_Of(&real_finding)] },
        ..Command_At(Repository_Root())
    };

    let result = Ran_Over(vec![source()], &command);

    assert!(!result.findings.suppressed_findings.is_empty(), "the double-matched finding must report as suppressed");
    assert!(
        result.findings.baselined_findings.is_empty(),
        "the double-matched finding must not also report as baselined: {:?}",
        result.findings.baselined_findings
    );
}

/// A [`crate::RuleCalibration`] matching the one blocking finding this fixture produces: the
/// run still judges the source and `check_outcome` still carries the finding in full, and it
/// now also appears in `calibrated_findings` rather than `blocking_findings` -- tolerated, not
/// silenced. Mirrors [`Test_A_Suppressed_Finding_Should_Not_Block`] and
/// [`Test_A_Baselined_Finding_Should_Not_Block`] for the third of `Run_Gate`'s three policies.
#[test]
fn Test_A_Calibrated_Finding_Should_Not_Block()
{
    let source = || return Mirrored_Source(SourcePath("a.rs"));
    let real_finding = One_Real_Blocking_Finding(source);
    let command = Command_With_Calibration(Repository_Root(), Calibration_Of(&real_finding));

    let result = Ran_Over(vec![source()], &command);

    Assert_Tolerated_Not_Blocking(&result, &result.findings.calibrated_findings);
}

/// A finding matched by a [`crate::RuleCalibration`], a [`crate::Suppression`] and a
/// [`crate::BaselineDebt`] all at once reports as calibrated, not suppressed or baselined --
/// `Run_Gate` checks calibration first, since it is a coarser, rule-wide override, so none of
/// the three lists double-count the same finding.
#[test]
fn Test_A_Calibrated_Suppressed_And_Baselined_Finding_Should_Report_As_Calibrated()
{
    let source = || return Mirrored_Source(SourcePath("a.rs"));
    let real_finding = One_Real_Blocking_Finding(source);
    let command = GateCommand {
        adoption: AdoptionPolicy { calibrated: vec![Calibration_Of(&real_finding)] },
        suppressions: SuppressionPolicy { suppressions: vec![Suppression_Of(&real_finding)] },
        baseline: BaselinePolicy { debt: vec![Baseline_Of(&real_finding)] },
        ..Command_At(Repository_Root())
    };

    let result = Ran_Over(vec![source()], &command);

    assert!(!result.findings.calibrated_findings.is_empty(), "the triple-matched finding must report as calibrated");
    assert!(
        result.findings.suppressed_findings.is_empty(),
        "the triple-matched finding must not also report as suppressed: {:?}",
        result.findings.suppressed_findings
    );
    assert!(
        result.findings.baselined_findings.is_empty(),
        "the triple-matched finding must not also report as baselined: {:?}",
        result.findings.baselined_findings
    );
}
