//! Every rule a real run should treat as advisory rather than blocking, regardless of
//! per-finding suppression or baseline debt.

use super::RuleCalibration;
use nomos_contracts::Finding;

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
        return self.calibrated.iter().find(|entry| return entry.Is_Applicable_To(finding));
    }
}
