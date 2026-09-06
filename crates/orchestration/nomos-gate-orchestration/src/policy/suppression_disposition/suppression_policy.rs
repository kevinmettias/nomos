//! Every disposition a run should honor.

use super::Suppression;
use nomos_contracts::Finding;

/// Every disposition a run should honor.
///
/// Empty is "nothing is suppressed": `Default` gives that state, so every construction site
/// that predates this type is unchanged in behavior. It is also what
/// [`crate::Run_Gate`] reads as "take this from the `nomos-gate.json` under the run's root,
/// if there is one" -- see `crate::policy::gate_policy_file`. CI's own `gate run --root .`
/// is unchanged because this repository declares no such file.
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
        return self.suppressions.iter().find(|suppression| return suppression.Is_Applicable_To(finding));
    }
}

#[cfg(test)]
mod tests
{
    use super::{Suppression, SuppressionPolicy};
    use crate::policy::suppression_disposition::SuppressionDisposition;
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

    #[test]
    fn Test_Suppressing_Should_Find_The_Entry_That_Applies_To_A_Finding()
    {
        let finding = Finding_For("naming-convention");
        let suppression = Suppression {
            rule: finding.rule.clone(),
            subject: finding.subject,
            disposition: SuppressionDisposition::FalsePositiveDisposition,
            rationale: "known false positive".to_owned(),
            owner: "author".to_owned(),
        };
        let policy = SuppressionPolicy { suppressions: vec![suppression.clone()] };

        assert_eq!(policy.Suppressing(&finding), Some(&suppression));
    }

    fn Finding_For(rule: &str) -> Finding
    {
        return Finding {
            rule: RuleId::New(rule),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
            subject_name: "Example".to_owned(),
            applicability: Applicability::Supported,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Blocking,
            summary: "example".to_owned(),
            locations: vec!["a.rs".to_owned()],
        };
    }

    #[test]
    fn Test_Suppressing_Should_Find_Nothing_When_No_Entry_Applies()
    {
        let policy = SuppressionPolicy::default();

        assert!(policy.Suppressing(&Finding_For("naming-convention")).is_none());
    }
}
