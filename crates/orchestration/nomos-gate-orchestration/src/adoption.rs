//! Whether an adopting repository has calibrated a whole rule to advisory rather than
//! blocking, separate from tolerating any one finding.

use nomos_contracts::{Finding, RuleId};

/// One rule an adopting repository has declared not yet blocking.
///
/// Matched by `rule` alone, not `rule`/`subject` -- deliberately a different addressing
/// scheme than [`crate::Suppression`] and [`crate::BaselineDebt`]: `ADOPT-CONFIG-*` is a
/// layer above both, "declared gates, phases and calibration policy," not a third way to
/// name one finding. A calibrated rule cannot fail a build regardless of which subject
/// triggered it, because calibration overrides a rule's category for this repository's
/// adoption stage entirely -- the corpus's "not by forking rule implementations" language,
/// expressed as configuration instead. No declared phases, thresholds or approvals here --
/// `ADOPT-CONFIG-*`'s full corpus shape -- and no owner/approver/date fields, the same
/// "nothing constructs one yet, so validating fields nothing populates would validate
/// against nothing" discipline [`crate::BaselineDebt`]'s own doc already states for this
/// crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleCalibration
{
    /// The rule this calibration applies to.
    pub rule: RuleId,
    /// Why this rule is not yet blocking for this repository -- required, the same reason
    /// [`crate::BaselineDebt::rationale`] and [`crate::Suppression::rationale`] both are.
    pub rationale: String,
}

impl RuleCalibration
{
    /// Whether this entry applies to `finding` -- by `rule` alone.
    #[must_use]
    pub fn Matches(&self, finding: &Finding) -> bool
    {
        return self.rule == finding.rule;
    }
}

/// Every rule a real run should treat as advisory rather than blocking, regardless of
/// per-finding suppression or baseline debt.
///
/// Empty is "nothing is calibrated," the state every caller is in today: `Default` gives
/// that state, so every construction site that predates this type and CI's own `gate run
/// --root .` are unchanged in behavior -- the same guarantee [`crate::SuppressionPolicy`]
/// and [`crate::BaselinePolicy`] both make.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AdoptionPolicy
{
    /// The calibrations this policy holds, in authoring order.
    pub calibrated: Vec<RuleCalibration>,
}

impl AdoptionPolicy
{
    /// The first entry that calibrates `finding`'s rule, if any.
    ///
    /// First rather than every match, the same "no invented rule nothing needs yet"
    /// discipline [`crate::SuppressionPolicy::Suppressing`] and [`crate::BaselinePolicy::
    /// Tolerating`] both already keep: two entries naming the same rule is an authoring
    /// question this increment does not referee.
    #[must_use]
    pub fn Calibrating<'a>(&'a self, finding: &Finding) -> Option<&'a RuleCalibration>
    {
        return self.calibrated.iter().find(|entry| return entry.Matches(finding));
    }
}

#[cfg(test)]
mod tests
{
    use super::{AdoptionPolicy, RuleCalibration};
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
    fn Test_An_Empty_Policy_Should_Calibrate_Nothing()
    {
        let policy = AdoptionPolicy::default();

        assert!(policy.Calibrating(&Finding_For("naming-convention", 1)).is_none());
    }

    #[test]
    fn Test_A_Matching_Rule_Should_Be_Found_Regardless_Of_Subject()
    {
        let finding = Finding_For("naming-convention", 1);
        let calibration = RuleCalibration { rule: RuleId::New("naming-convention"), rationale: "adopting incrementally".to_owned() };
        let policy = AdoptionPolicy { calibrated: vec![calibration.clone()] };

        assert_eq!(policy.Calibrating(&finding), Some(&calibration));
        assert_eq!(policy.Calibrating(&Finding_For("naming-convention", 2)), Some(&calibration));
    }

    #[test]
    fn Test_A_Mismatched_Rule_Should_Not_Match()
    {
        let finding = Finding_For("naming-convention", 1);
        let calibration = RuleCalibration { rule: RuleId::New("dependency-direction"), rationale: "different rule".to_owned() };
        let policy = AdoptionPolicy { calibrated: vec![calibration] };

        assert!(policy.Calibrating(&finding).is_none());
    }
}
