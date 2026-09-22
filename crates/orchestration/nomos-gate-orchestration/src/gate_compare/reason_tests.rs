//! The tests about the reason a finding was suppressed, and what a change of reason means.
//!
//! Split out of `gate_compare.rs` when that file passed the workspace's own review triggers.

use super::tests::{Bucketed_Findings, Comparison_Of, Result_With, BASELINE_RUN, CANDIDATE_RUN};
use crate::{
    FindingDisposition, GateFindings, SuppressionDisposition, SuppressionReason, SuppressionStatus,
};
use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// The subject the one finding this module records belongs to.
const SUBJECT_DIGEST: Digest128 = Digest128::From_Bytes([7; Digest128::BYTE_LENGTH]);

/// A reason changing inside one bucket is a change.
///
/// The transition this whole increment exists for. `FalsePositiveDisposition` says the
/// rule was wrong here; `FormalRiskAcceptance` says somebody owns the risk. The bucket is
/// `Suppressed` on both sides and the engineering claim is opposite, and before this the
/// comparison reported nothing at all.
#[test]
fn Test_A_Disposition_Change_Within_Suppressed_Should_Be_Reported()
{
    let before = (FindingDisposition::Suppressed, Some(Suppression_Reason(SuppressionDisposition::FalsePositiveDisposition, SuppressionStatus::Active)));
    let after = (FindingDisposition::Suppressed, Some(Suppression_Reason(SuppressionDisposition::FormalRiskAcceptance, SuppressionStatus::Active)));

    let result = Gate_Comparison(before, after);
    let change = result.changed.first().expect("a reason change is a change");

    assert_eq!(change.before, FindingDisposition::Suppressed, "the bucket did not move and must not appear to");
    assert_eq!(change.after, FindingDisposition::Suppressed);
    assert_eq!(change.before_reason.expect("a reason").disposition, SuppressionDisposition::FalsePositiveDisposition);
    assert_eq!(change.after_reason.expect("a reason").disposition, SuppressionDisposition::FormalRiskAcceptance);
}

/// A waiver lapsing is a change, and the reason says so.
///
/// Under `P103` an expired waiver does not suppress, so the finding leaves the bucket.
/// Without the recorded reason a reader would see only `Suppressed -> Blocking` and could
/// not tell a tolerance coming due from a suppression being withdrawn by hand.
#[test]
fn Test_A_Waiver_Lapsing_Should_Be_Reported_With_Its_Reason()
{
    let before = (FindingDisposition::Suppressed, Some(Suppression_Reason(SuppressionDisposition::TemporaryWaiver, SuppressionStatus::Active)));
    let after = (FindingDisposition::Blocking, Some(Suppression_Reason(SuppressionDisposition::TemporaryWaiver, SuppressionStatus::Expired)));

    let result = Gate_Comparison(before, after);
    let change = result.changed.first().expect("a lapsed waiver is a change");

    assert_eq!(change.before, FindingDisposition::Suppressed);
    assert_eq!(change.after, FindingDisposition::Blocking);
    assert_eq!(change.after_reason.expect("a reason").status, SuppressionStatus::Expired);
}

/// A bucket change with no suppression on either side still reports.
#[test]
fn Test_Baselined_Becoming_Blocking_Should_Be_Reported()
{
    let result = Gate_Comparison((FindingDisposition::Baselined, None), (FindingDisposition::Blocking, None));
    let change = result.changed.first().expect("a bucket change is a change");

    assert_eq!((change.before, change.after), (FindingDisposition::Baselined, FindingDisposition::Blocking));
    assert!(change.before_reason.is_none() && change.after_reason.is_none());
}

/// Nothing moving reports nothing.
///
/// The converse control. Without it a comparison that reported every finding every time
/// would satisfy all three above and tell a reader nothing, which is the failure mode the
/// added/removed/changed split exists to avoid.
#[test]
fn Test_An_Unchanged_Bucket_And_Reason_Should_Report_No_Change()
{
    let reason = Some(Suppression_Reason(SuppressionDisposition::InlineSuppression, SuppressionStatus::Active));

    let result = Gate_Comparison((FindingDisposition::Suppressed, reason), (FindingDisposition::Suppressed, reason));

    assert!(
        result.changed.is_empty(),
        "a finding whose bucket and reason both held still reported as changed: {:?}",
        result.changed
    );
    assert!(result.added.is_empty() && result.removed.is_empty());
}

fn Finding_Here() -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New("naming-convention"),
        subject: SubjectId::From_Digest(SUBJECT_DIGEST),
        subject_name: "src/lib.rs".to_string(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: "a name".to_string(),
        locations: Vec::new(),
    };
}

fn Suppression_Reason(disposition: SuppressionDisposition, status: SuppressionStatus) -> SuppressionReason
{
    return SuppressionReason { disposition, status };
}

/// `findings` with one finding in `bucket`, and `reason` recorded against it.
fn Findings_With(bucket: FindingDisposition, reason: Option<SuppressionReason>) -> GateFindings
{
    let finding = Finding_Here();
    let mut findings = Bucketed_Findings(&finding, bucket);

    if let Some(reason) = reason
    {
        findings.suppression_reasons.insert((finding.rule.clone(), finding.subject), reason);
    }

    return findings;
}

fn Gate_Comparison(
    before: (FindingDisposition, Option<SuppressionReason>),
    after: (FindingDisposition, Option<SuppressionReason>),
) -> super::GateCompareResult
{
    let baseline_findings = Findings_With(before.0, before.1);
    let candidate_findings = Findings_With(after.0, after.1);

    let baseline = Result_With(BASELINE_RUN, baseline_findings);
    let candidate = Result_With(CANDIDATE_RUN, candidate_findings);

    return Comparison_Of(&baseline, &candidate);
}
