//! One disposition over one rule's finding on one subject.

use super::SuppressionDisposition;
use nomos_contracts::{Finding, RuleId, SubjectId};

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
    pub fn Is_Applicable_To(&self, finding: &Finding) -> bool
    {
        return self.rule == finding.rule && self.subject == finding.subject;
    }
}

#[cfg(test)]
mod tests
{
    use super::{Suppression, SuppressionDisposition};
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

    #[test]
    fn Test_Is_Applicable_To_Should_Match_Same_Rule_And_Subject()
    {
        let finding = Finding_For("naming-convention", 1);
        let suppression = Suppression_Of(&finding);

        assert!(suppression.Is_Applicable_To(&finding));
    }

    #[test]
    fn Test_Is_Applicable_To_Should_Not_Match_A_Different_Subject()
    {
        let finding = Finding_For("naming-convention", 1);
        let suppression = Suppression_Of(&Finding_For("naming-convention", 2));

        assert!(!suppression.Is_Applicable_To(&finding));
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

    fn Suppression_Of(finding: &Finding) -> Suppression
    {
        return Suppression {
            rule: finding.rule.clone(),
            subject: finding.subject,
            disposition: SuppressionDisposition::FalsePositiveDisposition,
            rationale: "known false positive".to_owned(),
            owner: "author".to_owned(),
        };
    }
}
