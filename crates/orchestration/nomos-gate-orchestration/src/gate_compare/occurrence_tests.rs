//! The four behaviours occurrence scope changed, the refusal it introduced, and the two
//! addressings it deliberately left alone.
//!
//! Built against [`GateFindings`] directly rather than through a real policy change, the same
//! reason `reason_tests` states: what is under test is this module's own indexing, not a
//! second proof that a policy matches a finding.

use super::tests::{Comparison_Of, Result_With, BASELINE_RUN, CANDIDATE_RUN};
use crate::{
    Compare_Gate_Runs, FindingDisposition, GateFindings, SuppressionDisposition, SuppressionReason, SuppressionStatus,
};
use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};
use std::collections::BTreeMap;

/// The rule every occurrence in this module belongs to.
const RULE: &str = "nesting-depth";

/// The line numbers the fixtures below report occurrences at.
const FIRST_LINE: u32 = 10;
const SECOND_LINE: u32 = 20;
const THIRD_LINE: u32 = 30;
const FOURTH_LINE: u32 = 40;
const FIFTH_LINE: u32 = 50;

/// How many of the five occurrences the first fixture's candidate fixed.
const FIXED_OCCURRENCES: usize = 3;

/// How many more occurrences the second fixture's candidate reports than its baseline.
const ADDED_OCCURRENCES: usize = 4;

/// How many occurrences the two-occurrence fixtures move between buckets.
const MOVED_OCCURRENCES: usize = 2;

/// The one reason this module's suppression fixture records.
const ONE_REASON: SuppressionReason =
    SuppressionReason { disposition: SuppressionDisposition::FormalRiskAcceptance, status: SuppressionStatus::Active };

/// The subject every occurrence in this module belongs to.
const SUBJECT_DIGEST: Digest128 = Digest128::From_Bytes([5; Digest128::BYTE_LENGTH]);

/// Some of a subject's occurrences fixed, the rest still there.
///
/// Before occurrence scope this reported *nothing at all*: one finding before, one finding
/// after, same key, no difference. Three of five violations disappearing from a file is the
/// single most ordinary thing a comparison is asked about.
#[test]
fn Test_Fixing_Some_Occurrences_In_One_Subject_Should_Report_Removals()
{
    let baseline = Result_With(BASELINE_RUN, Blocking_At(&[FIRST_LINE, SECOND_LINE, THIRD_LINE, FOURTH_LINE, FIFTH_LINE]));
    let candidate = Result_With(CANDIDATE_RUN, Blocking_At(&[FIRST_LINE, SECOND_LINE]));

    let compared = Comparison_Of(&baseline, &candidate);

    assert_eq!(compared.removed.len(), FIXED_OCCURRENCES, "three of five were fixed: {:?}", compared.removed);
    assert!(compared.added.is_empty(), "{:?}", compared.added);
    assert!(compared.changed.is_empty(), "{:?}", compared.changed);
}

/// The same defect spreading through one file.
///
/// The other direction of the same silence, and the dangerous one: a file going from one
/// violation to five reported no change, so a regression inside an already-failing subject
/// was invisible.
#[test]
fn Test_Occurrences_Multiplying_In_One_Subject_Should_Report_Additions()
{
    let baseline = Result_With(BASELINE_RUN, Blocking_At(&[FIRST_LINE]));
    let candidate = Result_With(CANDIDATE_RUN, Blocking_At(&[FIRST_LINE, SECOND_LINE, THIRD_LINE, FOURTH_LINE, FIFTH_LINE]));

    let compared = Comparison_Of(&baseline, &candidate);

    assert_eq!(compared.added.len(), ADDED_OCCURRENCES, "four more appeared: {:?}", compared.added);
    assert!(compared.removed.is_empty(), "{:?}", compared.removed);
    assert!(compared.changed.is_empty(), "{:?}", compared.changed);
}

/// The vacuity guard, at the size that matters.
///
/// A comparison that finds something whenever a subject holds several occurrences would make
/// every one of the assertions above pass while being useless.
#[test]
fn Test_An_Unchanged_Multi_Occurrence_Subject_Should_Report_No_Change()
{
    let occurrences = Blocking_At(&[FIRST_LINE, SECOND_LINE, THIRD_LINE]);

    let baseline = Result_With(BASELINE_RUN, occurrences.clone());
    let candidate = Result_With(CANDIDATE_RUN, occurrences);

    let compared = Comparison_Of(&baseline, &candidate);

    assert!(compared.added.is_empty(), "{:?}", compared.added);
    assert!(compared.removed.is_empty(), "{:?}", compared.removed);
    assert!(compared.changed.is_empty(), "{:?}", compared.changed);
}

/// The population that already worked still works.
///
/// One finding at one subject was never the broken case, and a fix that changed its answer
/// would have traded one silent wrong report for another.
#[test]
fn Test_A_Single_Finding_Subject_Should_Behave_As_It_Did()
{
    let baseline = Result_With(BASELINE_RUN, Blocking_At(&[FIRST_LINE]));
    let candidate = Result_With(CANDIDATE_RUN, Blocking_At(&[]));

    let compared = Comparison_Of(&baseline, &candidate);

    assert_eq!(compared.removed.len(), 1, "{:?}", compared.removed);
    assert!(compared.added.is_empty(), "{:?}", compared.added);
}

/// A run holding two findings that cannot be told apart is refused, and both are named.
///
/// The negative control for the whole mechanism. These two differ only in gate category,
/// which the occurrence identity excludes by construction, so they reach one identity -- the
/// exact limit [`nomos_model::FindingOccurrenceId`]'s own documentation states. Without this
/// assertion the validation is only known to pass on populations that already satisfy it,
/// which is indistinguishable from a validation that cannot fail.
#[test]
fn Test_A_Run_Whose_Findings_Collide_Should_Refuse_And_Name_Both()
{
    let blocked = Occurrence_At(FIRST_LINE);
    let advisory = Finding { gate: GateCategory::Advisory, ..Occurrence_At(FIRST_LINE) };

    let baseline = Result_With(BASELINE_RUN, Colliding_Findings());
    let candidate = Result_With(CANDIDATE_RUN, Blocking_At(&[FIRST_LINE]));

    let refused = Compare_Gate_Runs(&baseline, &candidate).expect_err("two findings share one identity");

    assert_eq!(refused.run, BASELINE_RUN, "the refusal must say which side could not be indexed");
    assert_eq!(refused.collisions.len(), 1, "{:?}", refused.collisions);

    let collision = refused.collisions.first().expect("asserted len 1 above");

    assert_eq!(collision.first, blocked, "the collision names the finding that got there first");
    assert_eq!(collision.second, advisory, "and the one that collided with it");
}

/// A population holding two findings that reach one occurrence identity.
fn Colliding_Findings() -> GateFindings
{
    let mut findings = Blocking_At(&[FIRST_LINE]);

    findings.calibrated_findings.push(Finding { gate: GateCategory::Advisory, ..Occurrence_At(FIRST_LINE) });

    return findings;
}

/// A clean run is not refused.
///
/// The other direction of the refusal: a validation that rejected everything would satisfy
/// the test above and destroy the verb.
#[test]
fn Test_A_Run_Whose_Findings_Are_Distinct_Should_Not_Be_Refused()
{
    let occurrences = Blocking_At(&[FIRST_LINE, SECOND_LINE, THIRD_LINE]);

    let baseline = Result_With(BASELINE_RUN, occurrences.clone());
    let candidate = Result_With(CANDIDATE_RUN, occurrences);

    assert!(Compare_Gate_Runs(&baseline, &candidate).is_ok());
}

/// One suppression entry still covers every occurrence at its subject.
///
/// **The invariant this increment must not break.** Comparison moved to occurrence scope;
/// suppression addressing did not. `suppression_reasons` is keyed by (`rule`, `subject`), so
/// a single recorded reason has to reach both occurrences below. If `Reason_For` were looking
/// up by occurrence identity instead, one of these two changes would carry no reason and a
/// repository's existing suppressions would have silently narrowed to one line of one
/// revision.
#[test]
fn Test_A_Suppression_Reason_Should_Reach_Every_Occurrence_At_Its_Subject()
{
    let baseline = Result_With(BASELINE_RUN, Blocking_At(&[FIRST_LINE, SECOND_LINE]));
    let candidate = Result_With(CANDIDATE_RUN, Suppressed_Both());

    let compared = Comparison_Of(&baseline, &candidate);

    assert_eq!(compared.changed.len(), MOVED_OCCURRENCES, "both occurrences moved bucket: {:?}", compared.changed);

    for change in &compared.changed
    {
        assert_eq!(change.before, FindingDisposition::Blocking);
        assert_eq!(change.after, FindingDisposition::Suppressed);
        assert_eq!(
            change.after_reason,
            Some(ONE_REASON),
            "one suppression entry addresses a rule and a subject, so it must reach every \
             occurrence there -- narrowing it to one occurrence is a policy change this \
             increment does not make"
        );
    }
}

/// Both of a subject's two occurrences suppressed, with one reason recorded against the
/// subject rather than against either occurrence.
fn Suppressed_Both() -> GateFindings
{
    let mut reasons = BTreeMap::new();
    reasons.insert((RuleId::New(RULE), Subject()), ONE_REASON);

    return GateFindings {
        blocking_findings: Vec::new(),
        calibrated_findings: Vec::new(),
        suppressed_findings: vec![Occurrence_At(FIRST_LINE), Occurrence_At(SECOND_LINE)],
        baselined_findings: Vec::new(),
        baseline_exceeded_findings: Vec::new(),
        below_evidence_floor_findings: Vec::new(),
        baseline_populations: Vec::new(),
        suppression_reasons: reasons,
    };
}

/// Two changes at one subject are told apart by their geometry.
///
/// Occurrence scope made (`rule`, `subject_name`) non-unique in `changed`, and the rendered
/// line is built from those two. Without `locations` a reader would see the same line twice
/// and have no way to know it described two different places.
#[test]
fn Test_Two_Changes_At_One_Subject_Should_Be_Distinguishable()
{
    let baseline = Result_With(BASELINE_RUN, Blocking_At(&[FIRST_LINE, SECOND_LINE]));
    let candidate = Result_With(CANDIDATE_RUN, Calibrated_Both());

    let compared = Comparison_Of(&baseline, &candidate);

    let locations: Vec<&Vec<String>> = compared.changed.iter().map(|change| return &change.locations).collect();

    assert_eq!(compared.changed.len(), MOVED_OCCURRENCES, "{:?}", compared.changed);
    assert_ne!(locations.first(), locations.last(), "two changes at one subject must not render identically");
}

/// Both of a subject's two occurrences moved into the calibrated bucket, so the two changes
/// are told apart by their geometry alone.
fn Calibrated_Both() -> GateFindings
{
    return GateFindings {
        blocking_findings: Vec::new(),
        calibrated_findings: vec![Occurrence_At(FIRST_LINE), Occurrence_At(SECOND_LINE)],
        suppressed_findings: Vec::new(),
        baselined_findings: Vec::new(),
        baseline_exceeded_findings: Vec::new(),
        below_evidence_floor_findings: Vec::new(),
        baseline_populations: Vec::new(),
        suppression_reasons: BTreeMap::new(),
    };
}

fn Subject() -> SubjectId
{
    return SubjectId::From_Digest(SUBJECT_DIGEST);
}

/// One occurrence of [`RULE`] in `src/deep.rs`, at `line`.
///
/// Every field except the location is identical across occurrences, which is exactly the
/// population that used to collapse: one rule, one subject, several places. `subject_name` is
/// the file rather than the line on purpose -- it is the coarse spelling most rules use, and
/// a fixture that varied it would hide the collapse behind a field the identity does not read.
fn Occurrence_At(line: u32) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(RULE),
        subject: Subject(),
        subject_name: "src/deep.rs".to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: "nested too deeply".to_owned(),
        locations: vec![format!("src/deep.rs:{line}")],
    };
}

/// One occurrence per entry of `lines`, all in the blocking bucket.
fn Blocking_At(lines: &[u32]) -> GateFindings
{
    return GateFindings {
        blocking_findings: lines.iter().map(|line| return Occurrence_At(*line)).collect(),
        calibrated_findings: Vec::new(),
        suppressed_findings: Vec::new(),
        baselined_findings: Vec::new(),
        baseline_exceeded_findings: Vec::new(),
        below_evidence_floor_findings: Vec::new(),
        baseline_populations: Vec::new(),
        suppression_reasons: BTreeMap::new(),
    };
}
