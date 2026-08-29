//! What kind of sameness a strategy promises across repeated execution.

use serde::{Deserialize, Serialize};

const NONE_LABEL: &str = "None";
const STATE_LABEL: &str = "State";
const STATE_TEMPORAL_LABEL: &str = "StateTemporal";

/// Degree of determinism guaranteed by a strategy.
///
/// The distinction between [`DeterminismStrength::State`] and
/// [`DeterminismStrength::StateTemporal`] is the one that catches people out. A run
/// that produces the same set of findings has `State`. A run that also produces them in
/// the same order has `StateTemporal` — and for Nomos that stronger claim is what makes
/// a finding transcript diffable, which is what makes goldens possible at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DeterminismStrength
{
    /// No determinism claim. The honest declaration for anything reading a clock,
    /// sampling, or consuming a model backend.
    None,
    /// Final authoritative state is reproducible. The same inputs yield the same
    /// outputs; the order in which they were produced is not promised.
    State,
    /// Final state and the sequence of authoritative events are both reproducible.
    StateTemporal,
}

impl DeterminismStrength
{
    /// The variant's stable `PascalCase` name, for display and diagnostics.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::None => NONE_LABEL,
            Self::State => STATE_LABEL,
            Self::StateTemporal => STATE_TEMPORAL_LABEL,
        };
    }

    /// Whether this strength admits a reproducibility claim at all.
    ///
    /// Used to enforce the one cross-axis rule in the triple: a strategy claiming
    /// [`DeterminismStrength::None`] may not also claim a meaningful trace, and a
    /// strategy claiming `State` or stronger may not decline to define one.
    #[must_use]
    pub const fn Can_Claim_Reproducibility(self) -> bool
    {
        return matches!(self, Self::State | Self::StateTemporal);
    }
}

impl core::fmt::Display for DeterminismStrength
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_None_Should_Not_Claim_Reproducibility()
    {
        assert!(!DeterminismStrength::None.Can_Claim_Reproducibility());
        assert!(DeterminismStrength::State.Can_Claim_Reproducibility());
        assert!(DeterminismStrength::StateTemporal.Can_Claim_Reproducibility());
    }

    /// The ordering is load-bearing: a hierarchical strategy must declare the
    /// *weakest* strength among the strategies it can route to, and that comparison is
    /// written as `min`.
    #[test]
    fn Test_Strength_Should_Order_Weakest_First()
    {
        assert!(DeterminismStrength::None < DeterminismStrength::State);
        assert!(DeterminismStrength::State < DeterminismStrength::StateTemporal);
        assert_eq!(
            DeterminismStrength::StateTemporal.min(DeterminismStrength::None),
            DeterminismStrength::None
        );
    }
}
