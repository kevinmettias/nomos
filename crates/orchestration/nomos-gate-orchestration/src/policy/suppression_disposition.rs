//! A single disposition over one rule's finding on one subject.

mod suppression;
mod suppression_policy;
mod suppression_reason;
mod suppression_status;

pub use suppression::Suppression;
pub use suppression_policy::SuppressionPolicy;
pub use suppression_reason::SuppressionReason;
pub use suppression_status::SuppressionStatus;

/// The six dispositions `SUP-*` (the v14 corpus's `05.7-2-2 suppression and waiver
/// governance` section) names, not one generic "suppressed" bit -- a repository's reason
/// for disagreeing with a finding is part of what a reviewer needs to see, and collapsing
/// them would throw that away at the one point it is authored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuppressionDisposition
{
    /// Suppressed at the finding's own site, e.g. an inline annotation.
    InlineSuppression,
    /// Exempted by a repository-wide policy rather than a per-finding annotation.
    RepositoryPolicyException,
    /// Accepted for a bounded period, expected to be revisited.
    TemporaryWaiver,
    /// Existing debt a baseline tolerates rather than blocks -- distinct from a waiver,
    /// which names an end date this does not.
    AcceptedBaselineDebt,
    /// The finding does not hold; the rule (or its inputs) were wrong here.
    FalsePositiveDisposition,
    /// A deliberate, owned decision to accept the risk the finding names.
    FormalRiskAcceptance,
}

#[cfg(test)]
mod tests
{
    use super::{Suppression, SuppressionDisposition, SuppressionPolicy};
    use nomos_contracts::{Digest128, Finding, RuleId, SubjectId};
    use nomos_contracts::{Applicability, EvidenceClass, GateCategory};

    const DISTINCT_SEED_BYTE: u8 = 2;

    #[test]
    fn Test_Suppressing_Should_Report_Nothing_For_An_Empty_Policy()
    {
        let policy = SuppressionPolicy::default();

        assert!(policy.Suppressing(&Finding_For("naming-convention", 1), nomos_platform::Timestamp::From_Unix_Seconds(0)).is_none());
    }

    #[test]
    fn Test_Is_Applicable_To_Should_Match_Same_Rule_And_Subject()
    {
        let finding = Finding_For("naming-convention", 1);
        let suppression = Suppression {
            rule: RuleId::New("naming-convention"),
            subject: finding.subject,
            disposition: SuppressionDisposition::FalsePositiveDisposition,
            rationale: "known false positive on generated code".to_owned(),
            owner: "author".to_owned(),
            expiry: None
        };
        let policy = SuppressionPolicy { suppressions: vec![suppression.clone()] };

        assert_eq!(policy.Suppressing(&finding, nomos_platform::Timestamp::From_Unix_Seconds(0)), Some(&suppression));
    }

    #[test]
    fn Test_A_Mismatched_Subject_Should_Not_Match()
    {
        let finding = Finding_For("naming-convention", 1);
        let suppression = Suppression {
            rule: RuleId::New("naming-convention"),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([DISTINCT_SEED_BYTE; Digest128::BYTE_LENGTH])),
            disposition: SuppressionDisposition::TemporaryWaiver,
            rationale: "different subject".to_owned(),
            owner: "author".to_owned(),
            expiry: None
        };
        let policy = SuppressionPolicy { suppressions: vec![suppression] };

        assert!(policy.Suppressing(&finding, nomos_platform::Timestamp::From_Unix_Seconds(0)).is_none());
    }

    #[test]
    fn Test_A_Mismatched_Rule_Should_Not_Match()
    {
        let finding = Finding_For("naming-convention", 1);
        let suppression = Suppression {
            rule: RuleId::New("dependency-direction"),
            subject: finding.subject,
            disposition: SuppressionDisposition::TemporaryWaiver,
            rationale: "different rule".to_owned(),
            owner: "author".to_owned(),
            expiry: None
        };
        let policy = SuppressionPolicy { suppressions: vec![suppression] };

        assert!(policy.Suppressing(&finding, nomos_platform::Timestamp::From_Unix_Seconds(0)).is_none());
    }

    fn Finding_For(rule: &str, subject_seed: u8) -> Finding
    {
        return Finding {
            rule: RuleId::New(rule),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([subject_seed; Digest128::BYTE_LENGTH])),
            subject_name: "Example".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "example".to_owned(),
            locations: vec!["a.rs".to_owned()],
        };
    }
}
