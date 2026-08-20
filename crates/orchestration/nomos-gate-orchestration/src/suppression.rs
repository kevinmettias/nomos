//! A single disposition over one rule's finding on one subject.

use nomos_contracts::{Finding, RuleId, SubjectId};

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

/// One disposition over one rule's finding on one subject.
///
/// `owner`/`approver`/dates/revalidation triggers are `SUP-*`'s other required fields and
/// are deliberately not here yet -- this increment's own crate doc says why: no caller
/// constructs a `Suppression` at all today, so enforcing fields nothing populates would be
/// validating against nothing. Adding them is a later increment once something authors one
/// for real.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suppression
{
    /// The rule this disposition applies to.
    pub rule: RuleId,
    /// The subject this disposition applies to.
    pub subject: SubjectId,
    /// Which of `SUP-*`'s six dispositions this is.
    pub disposition: SuppressionDisposition,
    /// Why -- required, because a disposition with no stated reason is not distinguishable
    /// from silence.
    pub rationale: String,
    /// Who is accountable for this disposition.
    pub owner: String,
}

impl Suppression
{
    /// Whether this disposition applies to `finding` -- the same `rule`/`subject` identity
    /// `Finding` already carries, so matching invents no addressing scheme of its own.
    #[must_use]
    pub fn Matches(&self, finding: &Finding) -> bool
    {
        return self.rule == finding.rule && self.subject == finding.subject;
    }
}

/// Every disposition a run should honor.
///
/// Empty is "nothing is suppressed," the state every caller is in today: `Default` gives
/// that state, so every construction site that predates this type and CI's own `gate run
/// --root .` are unchanged in behavior.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SuppressionPolicy
{
    /// The dispositions this policy holds, in authoring order.
    pub suppressions: Vec<Suppression>,
}

impl SuppressionPolicy
{
    /// The first disposition that applies to `finding`, if any.
    ///
    /// First rather than every match: two dispositions naming the same rule and subject is
    /// an authoring question this increment does not referee, the same "no invented rule
    /// nothing needs yet" discipline the rest of this crate already keeps.
    #[must_use]
    pub fn Suppressing<'a>(&'a self, finding: &Finding) -> Option<&'a Suppression>
    {
        return self.suppressions.iter().find(|suppression| return suppression.Matches(finding));
    }
}

#[cfg(test)]
mod tests
{
    use super::{Suppression, SuppressionDisposition, SuppressionPolicy};
    use nomos_contracts::{Digest128, Finding, RuleId, SubjectId};
    use nomos_contracts::{Applicability, EvidenceClass, GateCategory};

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

    #[test]
    fn Test_An_Empty_Policy_Should_Suppress_Nothing()
    {
        let policy = SuppressionPolicy::default();

        assert!(policy.Suppressing(&Finding_For("naming-convention", 1)).is_none());
    }

    #[test]
    fn Test_A_Matching_Rule_And_Subject_Should_Be_Found()
    {
        let finding = Finding_For("naming-convention", 1);
        let suppression = Suppression {
            rule: RuleId::New("naming-convention"),
            subject: finding.subject,
            disposition: SuppressionDisposition::FalsePositiveDisposition,
            rationale: "known false positive on generated code".to_owned(),
            owner: "author".to_owned(),
        };
        let policy = SuppressionPolicy { suppressions: vec![suppression.clone()] };

        assert_eq!(policy.Suppressing(&finding), Some(&suppression));
    }

    #[test]
    fn Test_A_Mismatched_Subject_Should_Not_Match()
    {
        let finding = Finding_For("naming-convention", 1);
        let suppression = Suppression {
            rule: RuleId::New("naming-convention"),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([2; Digest128::BYTE_LENGTH])),
            disposition: SuppressionDisposition::TemporaryWaiver,
            rationale: "different subject".to_owned(),
            owner: "author".to_owned(),
        };
        let policy = SuppressionPolicy { suppressions: vec![suppression] };

        assert!(policy.Suppressing(&finding).is_none());
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
        };
        let policy = SuppressionPolicy { suppressions: vec![suppression] };

        assert!(policy.Suppressing(&finding).is_none());
    }
}
