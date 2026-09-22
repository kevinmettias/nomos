//! The seam between `nomos_gate_orchestration` and `nomos_contracts`, exercised from outside
//! the crate through public types only.
//!
//! `Disposition_Of_Findings` was previously exercised only by a test compiled INTO the crate
//! (`src/tests.rs`'s `#[cfg(test)] mod tests`), which reaches `nomos_contracts::Finding` as an
//! ordinary dependency and can still see this crate's own private items. That proves the two
//! work when one side is opened up; it does not prove the PUBLIC contract holds, which is the
//! only thing a real consumer can depend on. This file drives the same reduction through
//! nothing but `nomos_gate_orchestration`'s and `nomos_contracts`' own public surfaces,
//! compiled as a separate crate the way a real second adapter would see it.

use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};
use nomos_gate_orchestration::{Disposition_Of_Findings, GateRunOutcome};

/// The byte every finding below seeds its subject digest with. The value carries no meaning of
/// its own -- any seed but zero distinguishes one subject from another -- so it is named here
/// rather than spelled at the one place that reads it.
const SUBJECT_SEED: u8 = 7;

/// One finding, built so `gate` and `applicability` are the only knobs a caller of
/// `Disposition_Of_Findings` cares about -- everything else here is filler a reader can ignore.
fn Finding_With(gate: GateCategory, applicability: Applicability) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New("naming-convention"),
        subject: SubjectId::From_Digest(Digest128::From_Bytes([SUBJECT_SEED; Digest128::BYTE_LENGTH])),
        subject_name: "Example".to_owned(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate,
        summary: "example".to_owned(),
        locations: vec!["a.rs".to_owned()],
    };
}

/// The happy path across the boundary: no findings at all is a clean run.
#[test]
fn Test_Disposition_Of_Findings_Should_Pass_With_No_Findings()
{
    assert_eq!(Disposition_Of_Findings(&[]), GateRunOutcome::Passed);
}

/// A finding that cannot fail a build -- advisory, unreachable, or review -- must not flip the
/// disposition, the whole reason `Finding::Can_Fail_A_Build` exists rather than a bare
/// "any finding at all" check. Proven here through the public `Applicability`/`GateCategory`
/// enums a real caller constructs a `Finding` from, not through anything private to this crate.
#[test]
fn Test_Disposition_Of_Findings_Should_Pass_When_Nothing_Can_Fail_A_Build()
{
    let findings = vec![
        Finding_With(GateCategory::Advisory, Applicability::Supported),
        Finding_With(GateCategory::Review, Applicability::Supported),
        Finding_With(GateCategory::Unreachable, Applicability::Supported),
        Finding_With(GateCategory::Blocking, Applicability::NotApplicable),
    ];

    assert_eq!(Disposition_Of_Findings(&findings), GateRunOutcome::Passed);
}

/// The one condition that must flip the disposition: a rule whose enforcer is honored, judging
/// a subject it actually reached -- the error that crosses this boundary in the other
/// direction from the happy path above.
#[test]
fn Test_Disposition_Of_Findings_Should_Fail_On_A_Blocking_Finding()
{
    let findings = vec![Finding_With(GateCategory::Blocking, Applicability::Supported)];

    assert_eq!(Disposition_Of_Findings(&findings), GateRunOutcome::Failed);
}

/// Any blocking finding fails the run, even beside findings that would not have -- the
/// lifecycle a caller imposes by handing this function a mixed, real-shaped list rather than
/// one finding at a time.
#[test]
fn Test_Disposition_Of_Findings_Should_Fail_With_One_Blocking_Finding_Among_Many()
{
    let findings = vec![
        Finding_With(GateCategory::Advisory, Applicability::Supported),
        Finding_With(GateCategory::Blocking, Applicability::Supported),
        Finding_With(GateCategory::Review, Applicability::Supported),
    ];

    assert_eq!(Disposition_Of_Findings(&findings), GateRunOutcome::Failed);
}
