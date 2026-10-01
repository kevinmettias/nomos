//! What a `nomos gate` run produced.

use crate::RuleCompositionError;

use crate::gate_plan::GatePlan;

/// What a `nomos gate` run produced.
pub enum GateOutcome
{
    /// The plan: what this gate's rule registry holds.
    Planned(GatePlan),
    /// This crate's own rule composition is self-contradictory -- a defect in the
    /// composition, not in anything a caller supplied. Not reachable today; see
    /// [`crate::composition::Registered`]'s own doc.
    Contradictory(RuleCompositionError),
}
