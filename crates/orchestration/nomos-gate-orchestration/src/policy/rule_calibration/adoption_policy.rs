//! Every rule a real run should treat as advisory rather than blocking, regardless of
//! per-finding suppression or baseline debt.

use super::RuleCalibration;
use nomos_contracts::Finding;

/// Every rule a real run should treat as advisory rather than blocking, regardless of
/// per-finding suppression or baseline debt.
///
/// Empty is "nothing is calibrated": `Default` gives that state, so every construction site
/// that predates this type is unchanged in behavior, and [`crate::Run_Gate`] reads it as
/// "take this from the `nomos-gate.json` under the run's root, if there is one" -- the same
/// guarantee and the same declared source [`crate::SuppressionPolicy`] and
/// [`crate::BaselinePolicy`] both have. A calibration matches by rule alone, so a
/// file-authored one reaches every rule rather than only the file-addressed ones.
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
        return self.calibrated.iter().find(|entry| return entry.Is_Applicable_To(finding));
    }
}

#[cfg(test)]
mod tests
{
    use super::{AdoptionPolicy, RuleCalibration};
    use nomos_contracts::{Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

    #[test]
    fn Test_Calibrating_Should_Find_The_Entry_That_Applies_To_A_Finding()
    {
        let finding = Finding_For("naming-convention");
        let calibration = RuleCalibration { rule: finding.rule.clone(), rationale: "adopting".to_owned() };
        let policy = AdoptionPolicy { calibrated: vec![calibration.clone()] };

        assert_eq!(policy.Calibrating(&finding), Some(&calibration));
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
    fn Test_Calibrating_Should_Find_Nothing_When_No_Entry_Applies()
    {
        let policy = AdoptionPolicy::default();

        assert!(policy.Calibrating(&Finding_For("naming-convention")).is_none());
    }
}
