//! [`JudgmentDifferenceResponse`], one judgment input a comparison found differing.

use nomos_gate_orchestration::JudgmentDifference;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::JudgmentDifference`].
///
/// A twin rather than a re-export for the reason [`super`]'s own doc gives for every other one
/// in this module.
///
/// # Why the source is not a variant
///
/// The source is absent, and that is the point of the whole type: a difference in source is
/// the licensed cause a comparison exists to attribute a change to, not a caveat on doing so.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JudgmentDifferenceResponse
{
    /// The two runs judged under different policies. The likeliest of the three and the least
    /// visible: a run resolves its policy from its own root, and a comparison judges two.
    Policy,
    /// The two runs were allowed to look at different things. A run told to look at less has
    /// fewer findings for that reason, and its absent findings otherwise read as the other
    /// run's additions.
    Selection,
    /// The two runs were judged by different builds or different rule sets. This workspace
    /// declares its analysis kernel reproducible `CrossPlatform`, which is strictly weaker
    /// than `CrossBinary`, so agreement across two instruments is not something it claims.
    Instrument,
}

impl JudgmentDifferenceResponse
{
    pub(crate) const fn From(difference: JudgmentDifference) -> Self
    {
        return match difference
        {
            JudgmentDifference::Policy => Self::Policy,
            JudgmentDifference::Selection => Self::Selection,
            JudgmentDifference::Instrument => Self::Instrument,
        };
    }
}
