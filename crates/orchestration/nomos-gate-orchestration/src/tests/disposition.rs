//! What one finding does to a whole run's verdict, before any policy is consulted.

use crate::{Disposition_Of_Findings, GateRunOutcome};
use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};
use nomos_rules::COMPLETENESS_MIRROR;

/// The byte every fixture below seeds its subject digest from.
///
/// Named rather than a bare fill at the construction site, because a digest built from a digit
/// says nothing about which subject it is and the number itself is a fixture's identity rather
/// than a quantity anything computes.
const SUBJECT_SEED: u8 = 7;

/// One finding, built so `gate` and `applicability` are the only knobs a caller of
/// [`Disposition_Of_Findings`] cares about -- everything else here is filler a reader can ignore.
fn Finding_With(gate: GateCategory, applicability: Applicability) -> Finding
{
    return Finding {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        subject: SubjectId::From_Digest(Digest128::From_Bytes([SUBJECT_SEED; Digest128::BYTE_LENGTH])),
        subject_name: "Example".to_owned(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate,
        summary: "example".to_owned(),
        locations: vec!["a.rs".to_owned()],
    };
}

/// No findings at all is `Passed`, the same "an empty judgment is clean, not unknown"
/// reading `nomos_check_orchestration::Claim_Of(&[])` already gives `Claim::Complete`.
#[test]
fn Test_No_Findings_Should_Pass()
{
    assert_eq!(Disposition_Of_Findings(&[]), GateRunOutcome::Passed);
}

/// A finding that cannot fail a build -- advisory, unreachable, or review -- must not flip
/// the disposition, the whole reason `Finding::Can_Fail_A_Build` exists rather than a bare
/// "any finding at all" check.
#[test]
fn Test_A_Non_Blocking_Finding_Should_Pass()
{
    let findings = vec![
        Finding_With(GateCategory::Advisory, Applicability::Supported),
        Finding_With(GateCategory::Review, Applicability::Supported),
        Finding_With(GateCategory::Unreachable, Applicability::Supported),
        Finding_With(GateCategory::Blocking, Applicability::NotApplicable),
    ];

    assert_eq!(Disposition_Of_Findings(&findings), GateRunOutcome::Passed);
}

/// The one condition that must flip the disposition: a rule whose enforcer is honored,
/// judging a subject it actually reached.
#[test]
fn Test_A_Blocking_Finding_Should_Fail()
{
    let findings = vec![Finding_With(GateCategory::Blocking, Applicability::Supported)];

    assert_eq!(Disposition_Of_Findings(&findings), GateRunOutcome::Failed);
}

/// Any blocking finding fails the run, even beside findings that would not have.
#[test]
fn Test_One_Blocking_Finding_Among_Many_Should_Fail()
{
    let findings = vec![
        Finding_With(GateCategory::Advisory, Applicability::Supported),
        Finding_With(GateCategory::Blocking, Applicability::Supported),
        Finding_With(GateCategory::Review, Applicability::Supported),
    ];

    assert_eq!(Disposition_Of_Findings(&findings), GateRunOutcome::Failed);
}
