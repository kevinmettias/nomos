//! [`SuppressionStanding`], whether a disposition still applied when a run was judged.

use nomos_gate_orchestration::SuppressionStatus;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::SuppressionStatus`].
///
/// Two variants because two is the whole population: a third, for a disposition stale by rule
/// version, subject identity, evidence or scope, is named in that type's own documentation and
/// deliberately not invented before something can construct it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionStanding
{
    /// The disposition applied when the run was judged.
    Active,
    /// It named an end date and the run was at or past it.
    Expired,
}

impl SuppressionStanding
{
    pub(crate) fn From(status: SuppressionStatus) -> Self
    {
        return match status
        {
            SuppressionStatus::Active => Self::Active,
            SuppressionStatus::Expired => Self::Expired,
        };
    }
}
