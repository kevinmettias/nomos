//! What a run records about why a finding did or did not block.

use super::reduction::{DispositionPolicies, Partitioned_Findings};
use crate::{AdoptionPolicy, BaselinePolicy, Suppression, SuppressionDisposition, SuppressionPolicy, SuppressionStatus};
use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};
use nomos_platform::Timestamp;

/// When the waiver below lapses, in unix seconds.
///
/// A constant rather than a literal at each use, because the three tests differ only in
/// whether they read before or at it, and a number spelled three times is a number a reader
/// has to compare three times.
const EXPIRY: i64 = 1_000;

/// The byte [`Finding_Here`] seeds its subject digest from.
///
/// Named because a digest built from a bare digit says nothing about which subject it is,
/// and the waiver below has to name the same one.
const SUBJECT_SEED: u8 = 3;

fn Waiver() -> Suppression
{
    let finding = Finding_Here();

    return Suppression {
        rule: finding.rule,
        subject: finding.subject,
        disposition: SuppressionDisposition::TemporaryWaiver,
        rationale: "bounded".to_string(),
        owner: "someone".to_string(),
        expiry: Some(Timestamp::From_Unix_Seconds(EXPIRY)),
    };
}

/// A live waiver suppresses, and its reason is recorded as active.
///
/// The forward direction: a suppressed finding always has a reason, so a reader is never
/// told a finding did not block without being told why.
#[test]
fn Test_A_Suppressed_Finding_Should_Carry_An_Active_Reason()
{
    let findings = Partitioned_At(EXPIRY - 1);
    let finding = findings.suppressed_findings.first().expect("a live waiver suppresses");
    let reason = findings
        .suppression_reasons
        .get(&(finding.rule.clone(), finding.subject))
        .expect("a suppressed finding must say why");

    assert_eq!(reason.disposition, SuppressionDisposition::TemporaryWaiver);
    assert_eq!(reason.status, SuppressionStatus::Active);
}

/// A lapsed waiver does not suppress, and its reason is still recorded as expired.
///
/// The direction the narrower invariant would have forbidden. Recording only suppressing
/// dispositions would leave this finding in `blocking_findings` with nothing saying a
/// tolerance came due rather than a violation appearing -- which is exactly the
/// distinction expiry was made visible for, lost again at the moment a reader looks for
/// it. So the recorded set is not the suppressed set, deliberately, and this is the case
/// that shows why.
#[test]
fn Test_A_Lapsed_Waiver_Should_Be_Recorded_Though_It_Did_Not_Suppress()
{
    let findings = Partitioned_At(EXPIRY);
    let finding = findings.blocking_findings.first().expect("a lapsed waiver does not suppress");

    assert!(findings.suppressed_findings.is_empty(), "an expired waiver must not suppress");

    let reason = findings
        .suppression_reasons
        .get(&(finding.rule.clone(), finding.subject))
        .expect("a finding blocking because a waiver lapsed must say so");

    assert_eq!(reason.status, SuppressionStatus::Expired);
}

/// A finding no disposition names carries no reason.
///
/// The converse control. Without it a recorder that attached a reason to everything would
/// satisfy both cases above and make the field meaningless.
#[test]
fn Test_A_Finding_No_Disposition_Names_Should_Carry_No_Reason()
{
    let adoption = AdoptionPolicy::default();
    let baseline = BaselinePolicy::default();
    let suppressions = SuppressionPolicy::default();

    let findings = Partitioned_Findings(
        &[Finding_Here()],
        DispositionPolicies {
            adoption: &adoption,
            suppressions: &suppressions,
            baseline: &baseline,
            now: Timestamp::From_Unix_Seconds(0),
        },
    );

    assert!(
        findings.suppression_reasons.is_empty(),
        "a finding nothing suppressed carried a reason: {:?}",
        findings.suppression_reasons
    );
}

fn Finding_Here() -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New("naming-convention"),
        subject: SubjectId::From_Digest(Digest128::From_Bytes([SUBJECT_SEED; Digest128::BYTE_LENGTH])),
        subject_name: "src/lib.rs".to_string(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: "a name".to_string(),
        locations: Vec::new(),
    };
}

fn Partitioned_At(seconds: i64) -> crate::GateFindings
{
    let adoption = AdoptionPolicy::default();
    let baseline = BaselinePolicy::default();
    let suppressions = SuppressionPolicy { suppressions: vec![Waiver()] };

    return Partitioned_Findings(
        &[Finding_Here()],
        DispositionPolicies {
            adoption: &adoption,
            suppressions: &suppressions,
            baseline: &baseline,
            now: Timestamp::From_Unix_Seconds(seconds),
        },
    );
}
