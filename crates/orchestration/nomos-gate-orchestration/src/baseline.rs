//! Existing debt a repository has named and chosen to tolerate, rather than block.

use nomos_contracts::{Finding, RuleId, SubjectId};

/// One finding a repository has named as tolerated existing debt.
///
/// Matched by `rule`/`subject`, the same identity [`crate::Suppression`] matches by and
/// [`Finding`] already carries -- no second addressing scheme invented for the same
/// question. Deliberately narrower than `BASELINE-*`'s full shape: no new-code/diff
/// detection, no distinction from a reintroduced or safety-critical finding, and no
/// owner/approver/date fields -- the same "nothing constructs one yet, so validating
/// fields nothing populates would validate against nothing" discipline
/// [`crate::Suppression`]'s own doc already states for this crate. A named entry here is a
/// closed, specific piece of debt, not a scope a diff could grow or shrink; scoping by
/// source geometry and revision is a later increment's concern, once a real caller needs
/// one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaselineDebt
{
    /// The rule this debt applies to.
    pub rule: RuleId,
    /// The subject this debt applies to.
    pub subject: SubjectId,
    /// Why this finding is tolerated rather than fixed -- required, for the reason
    /// [`crate::Suppression::rationale`] is: a tolerance with no stated reason is not
    /// distinguishable from an oversight.
    pub rationale: String,
}

impl BaselineDebt
{
    /// Whether this entry applies to `finding` -- the same `rule`/`subject` identity
    /// [`crate::Suppression::Matches`] compares by.
    #[must_use]
    pub fn Matches(&self, finding: &Finding) -> bool
    {
        return self.rule == finding.rule && self.subject == finding.subject;
    }
}

/// Every piece of existing debt a real run should tolerate rather than block.
///
/// Empty is "nothing is baselined," the state every caller is in today: `Default` gives
/// that state, so every construction site that predates this type and CI's own `gate run
/// --root .` are unchanged in behavior -- the same guarantee [`crate::SuppressionPolicy`]
/// makes.
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
        return self.debt.iter().find(|entry| return entry.Matches(finding));
    }
}

#[cfg(test)]
mod tests
{
    use super::{BaselineDebt, BaselinePolicy};
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
    fn Test_An_Empty_Policy_Should_Tolerate_Nothing()
    {
        let policy = BaselinePolicy::default();

        assert!(policy.Tolerating(&Finding_For("naming-convention", 1)).is_none());
    }

    #[test]
    fn Test_A_Matching_Rule_And_Subject_Should_Be_Found()
    {
        let finding = Finding_For("naming-convention", 1);
        let debt = BaselineDebt {
            rule: RuleId::New("naming-convention"),
            subject: finding.subject,
            rationale: "pre-existing, tracked for later cleanup".to_owned(),
        };
        let policy = BaselinePolicy { debt: vec![debt.clone()] };

        assert_eq!(policy.Tolerating(&finding), Some(&debt));
    }

    #[test]
    fn Test_A_Mismatched_Subject_Should_Not_Match()
    {
        let finding = Finding_For("naming-convention", 1);
        let debt = BaselineDebt {
            rule: RuleId::New("naming-convention"),
            subject: SubjectId::From_Digest(Digest128::From_Bytes([2; Digest128::BYTE_LENGTH])),
            rationale: "different subject".to_owned(),
        };
        let policy = BaselinePolicy { debt: vec![debt] };

        assert!(policy.Tolerating(&finding).is_none());
    }

    #[test]
    fn Test_A_Mismatched_Rule_Should_Not_Match()
    {
        let finding = Finding_For("naming-convention", 1);
        let debt = BaselineDebt {
            rule: RuleId::New("dependency-direction"),
            subject: finding.subject,
            rationale: "different rule".to_owned(),
        };
        let policy = BaselinePolicy { debt: vec![debt] };

        assert!(policy.Tolerating(&finding).is_none());
    }
}
