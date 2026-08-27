//! Every disposition a run should honor.

use super::Suppression;
use nomos_contracts::Finding;

/// Every disposition a run should honor.
///
/// Empty is "nothing is suppressed," the state every caller is in today: `Default` gives
/// that state, so every construction site that predates this type and CI's own `gate run
/// --root .` are unchanged in behavior.
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
        return self.suppressions.iter().find(|suppression| return suppression.Matches(finding));
    }
}
