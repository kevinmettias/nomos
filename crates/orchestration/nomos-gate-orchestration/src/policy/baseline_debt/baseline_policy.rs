//! Every piece of existing debt a real run should tolerate rather than block.

use super::BaselineDebt;
use nomos_contracts::Finding;

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
        return self.debt.iter().find(|entry| return entry.Is_Applicable_To(finding));
    }
}
