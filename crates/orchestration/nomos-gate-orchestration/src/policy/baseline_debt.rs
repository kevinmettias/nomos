//! Existing debt a repository has named and chosen to tolerate, rather than block.

mod baseline_policy;

pub use baseline_policy::BaselinePolicy;

use nomos_contracts::{Finding, RuleId, SubjectId};

/// One finding a repository has named as tolerated existing debt.
///
/// Matched by `rule`/`subject`, the same identity [`crate::Suppression`] matches by and
/// [`Finding`] already carries -- no second addressing scheme invented for the same
/// question. Deliberately narrower than `BASELINE-*`'s full shape: no new-code/diff
/// detection, no distinction from a reintroduced or safety-critical finding, and no
/// owner/approver/date fields. That last omission is a choice against a real alternative
/// rather than an absence waiting on one: `P40-GATE-POLICY-AUTHORING-3` made a declared
/// `nomos-gate.json` the real constructor of these, and a debt entry there is exactly
/// `rule`, `path` and `rationale`. [`crate::Suppression`] is the contrast that shows the
/// difference is deliberate -- it carries an `owner` and the same file reads it -- so an
/// author who wants a debt attributed is being told no rather than overlooked, and
/// `deny_unknown_fields` makes writing one a refusal rather than a silent drop.
///
/// What would justify widening is a declared file that has to attribute or expire a
/// tolerance, in the order `Suppression::owner` already followed: the key appears on the
/// declared entry first, and this type grows a field to carry it. `allowance` is the first
/// field that arrived that way. A named entry here is a closed, specific piece of debt, not a
/// scope a diff could grow or shrink; scoping by source geometry and revision is a later
/// increment's concern, once a real caller needs one.
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
    /// How many occurrences this entry accepted at adoption.
    ///
    /// `OD-GATE-030`: an adopted baseline records the maximum occurrence population accepted
    /// for its scope, and a later run may tolerate no more than that quantity.
    pub allowance: BaselineAllowance,
}

/// How much debt one [`BaselineDebt`] accepted.
///
/// A state that names itself rather than an `Option<u32>` whose `None` a reader has to
/// interpret. The interpretation is load-bearing and counter-intuitive -- absence means
/// *unlimited*, not zero and not one -- so it is spelled, and every match over it has to say
/// which case it is handling.
///
/// # Why absence is unbounded rather than one
///
/// `OD-GATE-030` decides this and states the alternative it rejected. Every entry authored
/// before the quantity existed names none, and reading those as a single occurrence would
/// start blocking builds over debt a repository did adopt, with the gate claiming a number
/// nobody wrote. Reading them as unlimited keeps the meaning they were written under; what
/// makes that honest rather than a silent hole is that the state is *named*, so a run can
/// report the entry as unbounded instead of it being indistinguishable from a bounded one.
///
/// # What a count is not
///
/// It bounds capacity and establishes nothing about history. A scope that accepted five and
/// observes five is equally consistent with the same five persisting and with all five having
/// been fixed while five different violations appeared. `OD-GATE-030` says so at length, and
/// this type is deliberately not named for continuity, persistence or reintroduction so that
/// nobody reaches for it to answer one of those.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaselineAllowance
{
    /// The entry names no count, so it tolerates however many occurrences its scope holds.
    ///
    /// The state every entry authored before `OD-GATE-030` v2 is in, and the one a run reports
    /// rather than passes over in silence.
    Unbounded,
    /// The entry accepted at most this many occurrences.
    ///
    /// Never zero: an entry accepting none tolerates nothing, which is what declining to write
    /// the entry already does, so the declared reader refuses it rather than storing a value
    /// whose only effect would be to block what it claims to permit.
    AtMost(u32),
}

impl BaselineDebt
{
    /// Whether this entry applies to `finding` -- the same `rule`/`subject` identity
    /// [`crate::Suppression::Is_Applicable_To`] compares by.
    #[must_use]
    pub fn Is_Applicable_To(&self, finding: &Finding) -> bool
    {
        return self.rule == finding.rule && self.subject == finding.subject;
    }
}

#[cfg(test)]
mod tests
{
    use super::{BaselineAllowance, BaselineDebt, BaselinePolicy};
    use nomos_contracts::{Digest128, Finding, RuleId, SubjectId};
    use nomos_contracts::{Applicability, EvidenceClass, GateCategory};

    const DISTINCT_SEED_BYTE: u8 = 2;

    #[test]
    fn Test_Tolerating_Should_Report_Nothing_For_An_Empty_Policy()
    {
        let policy = BaselinePolicy::default();

        assert!(policy.Tolerating(&Finding_For("naming-convention", 1)).is_none());
    }

    #[test]
    fn Test_Is_Applicable_To_Should_Match_Same_Rule_And_Subject()
    {
        let finding = Finding_For("naming-convention", 1);
        let debt = BaselineDebt {
            rule: RuleId::New("naming-convention"),
            subject: finding.subject,
            rationale: "pre-existing, tracked for later cleanup".to_owned(),
            allowance: BaselineAllowance::Unbounded,
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
            subject: SubjectId::From_Digest(Digest128::From_Bytes([DISTINCT_SEED_BYTE; Digest128::BYTE_LENGTH])),
            rationale: "different subject".to_owned(),
            allowance: BaselineAllowance::Unbounded,
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
            allowance: BaselineAllowance::Unbounded,
        };
        let policy = BaselinePolicy { debt: vec![debt] };

        assert!(policy.Tolerating(&finding).is_none());
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
