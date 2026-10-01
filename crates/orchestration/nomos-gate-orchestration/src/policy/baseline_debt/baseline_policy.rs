//! Every piece of existing debt a real run should tolerate rather than block.

use super::BaselineDebt;
use nomos_contracts::Finding;

/// Every piece of existing debt a real run should tolerate rather than block.
///
/// Empty is "nothing is baselined": `Default` gives that state, so every construction site
/// that predates this type is unchanged in behavior, and [`crate::Run_Gate`] reads it as
/// "take this from the `nomos-gate.json` under the run's root, if there is one" -- the same
/// guarantee and the same declared source [`crate::SuppressionPolicy`] has.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BaselinePolicy
{
    /// The debt this policy holds, in authoring order.
    pub debt: Vec<BaselineDebt>,
}

impl BaselinePolicy
{
    /// The first entry that tolerates `finding`, if any.
    ///
    /// First rather than every match, the same "no invented rule nothing needs yet"
    /// discipline [`crate::SuppressionPolicy::Suppressing`] already keeps: two entries
    /// naming the same rule and subject is an authoring question this increment does not
    /// referee.
    #[must_use]
    pub fn Tolerating<'a>(&'a self, finding: &Finding) -> Option<&'a BaselineDebt>
    {
        return self.debt.iter().find(|entry| return entry.Is_Applicable_To(finding));
    }
}

#[cfg(test)]
mod tests
{
    use super::{BaselineDebt, BaselinePolicy};
    use crate::BaselineAllowance;
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

    #[test]
    fn Test_Tolerating_Should_Find_The_Entry_That_Applies_To_A_Finding()
    {
        let finding = Finding_For("naming-convention");
        let debt = BaselineDebt {
            rule: finding.rule.clone(),
            subject: finding.subject,
            rationale: "tracked".to_owned(),
            allowance: BaselineAllowance::Unbounded,
            declared_path: Some("src/lib.rs".to_owned()),
        };
        let policy = BaselinePolicy { debt: vec![debt.clone()] };

        assert_eq!(policy.Tolerating(&finding), Some(&debt));
    }

    fn Finding_For(rule: &str) -> Finding
    {
        return Finding {
            address: None,
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
    fn Test_Tolerating_Should_Find_Nothing_When_No_Entry_Applies()
    {
        let policy = BaselinePolicy::default();

        assert!(policy.Tolerating(&Finding_For("naming-convention")).is_none());
    }
}
