//! Existing debt a repository has named and chosen to tolerate, rather than block.

mod baseline_allowance;
mod baseline_policy;

pub use baseline_allowance::BaselineAllowance;
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
    /// The path an author wrote, when a declared `nomos-gate.json` is where this entry came
    /// from.
    ///
    /// **Display material, not identity.** `subject` is the identity and is what
    /// [`BaselineDebt::Is_Applicable_To`] compares; this is carried only so that a report can
    /// name a scope the way the person who wrote the entry named it. The mapping runs one
    /// way -- [`nomos_model::Subject_Of_Path`] folds a spelling into a digest and nothing
    /// recovers the spelling from it -- so a report without this can print the digest and
    /// nothing else, which is not what its reader wrote and not a string they can search their
    /// own configuration for.
    ///
    /// `None` when no file declared this entry, which is every policy a caller built in code.
    /// There is no authored spelling to carry in that case, and the digest is then genuinely
    /// all there is; the same `None` must not be invented for an entry whose file did name a
    /// path.
    ///
    /// **Nothing matches on this.** `./a.rs` and `a.rs` are two spellings of one subject, so
    /// two entries differing here can be the same scope -- and two spellings that normalize
    /// alike are exactly how that arises in a real file. A comparison that consulted this
    /// field would split one scope in two over a difference its author cannot see.
    pub declared_path: Option<String>,
}

impl BaselineDebt
{
    /// Whether this entry applies to `finding` -- the same `rule`/`subject` identity
    /// [`crate::Suppression::Is_Applicable_To`] compares by.
    ///
    /// `declared_path` is deliberately not consulted. It is display material rather than
    /// identity, and consulting it would make an entry's reach depend on how its author spelled
    /// a path rather than on which file they meant -- so two entries that name one file would
    /// tolerate different findings according to whether either wrote a `./` prefix.
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
    use nomos_model::Subject_Of_Path;

    const DISTINCT_SEED_BYTE: u8 = 2;

    /// What each of the two spellings below accepted at adoption.
    ///
    /// Two names rather than two bare numbers because the pair carries the test: the two entries
    /// differ in their allowance as well as in their spelling, so an implementation that matched
    /// on either field would still have to agree with itself about the other.
    const FIRST_SPELLING_ACCEPTS: u32 = 2;
    const SECOND_SPELLING_ACCEPTS: u32 = 5;

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
        let debt = Debt_Over(&finding, Some("src/lib.rs"), BaselineAllowance::Unbounded);
        let policy = BaselinePolicy { debt: vec![debt.clone()] };

        assert_eq!(policy.Tolerating(&finding), Some(&debt));
    }

    /// Two entries naming one file by two spellings are one scope, so both tolerate the same
    /// finding.
    ///
    /// The clause `P109-D` exists to hold: a declared path is display material and never
    /// identity, so an entry's reach cannot depend on whether its author wrote a `./` prefix,
    /// backslashes or capitals. `nomos_model::Subject_Of_Path` folds every one of those to one
    /// digest, which is what makes this reachable in a real `nomos-gate.json` rather than
    /// only in a test.
    ///
    /// This is the assertion that keeps the field from drifting into the matching path: an
    /// implementation that compared `declared_path` on the way past would satisfy every other
    /// test in this file and fail this one.
    #[test]
    fn Test_Two_Entries_Naming_One_File_By_Two_Spellings_Should_Tolerate_The_Same_Finding()
    {
        let finding = Finding_Subjected_To("src/lib.rs");
        let spelled_one_way = Debt_Over(&finding, Some("./src/lib.rs"), BaselineAllowance::AtMost(FIRST_SPELLING_ACCEPTS));
        let spelled_another = Debt_Over(&finding, Some("Src\\Lib.rs"), BaselineAllowance::AtMost(SECOND_SPELLING_ACCEPTS));

        assert_eq!(spelled_one_way.subject, spelled_another.subject, "one file, one subject");
        assert_ne!(spelled_one_way.declared_path, spelled_another.declared_path, "and two spellings, so this test is about the field it says it is");
        assert!(spelled_one_way.Is_Applicable_To(&finding), "the first spelling must still reach the finding");
        assert!(spelled_another.Is_Applicable_To(&finding), "and so must the second");
    }

    /// A finding whose subject is `path`'s, which is what a real walk files it under.
    ///
    /// [`Finding_For`] above seeds a digest by byte, which is enough for a test about two
    /// entries agreeing but says nothing about whether a *spelling* folds to it -- and folding
    /// is the whole claim the test above makes. So this one goes through the kernel's own
    /// mapping, the same function every real walker and `gate_policy_file` call.
    fn Finding_Subjected_To(path: &str) -> Finding
    {
        return Finding { subject: Subject_Of_Path(path), ..Finding_For("naming-convention", 1) };
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
            declared_path: Some("elsewhere.rs".to_owned()),
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
            declared_path: Some("src/lib.rs".to_owned()),
        };
        let policy = BaselinePolicy { debt: vec![debt] };

        assert!(policy.Tolerating(&finding).is_none());
    }

    /// One entry over `finding`'s own scope, as a declared file would have produced it.
    ///
    /// `allowance` is a parameter because the tests above differ in it, and `declared_path`
    /// because the whole point of carrying it is that it can differ while the scope does not.
    fn Debt_Over(finding: &Finding, declared_path: Option<&str>, allowance: BaselineAllowance) -> BaselineDebt
    {
        return BaselineDebt {
            rule: finding.rule.clone(),
            subject: finding.subject,
            rationale: "test fixture".to_owned(),
            allowance,
            declared_path: declared_path.map(str::to_owned),
        };
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
