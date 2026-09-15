//! One approval of a phase that would otherwise fail.

/// One approval of a phase that would otherwise fail -- `WF-001`'s "approvals" clause.
///
/// Matched by `phase` name alone, the same coarse, whole-phase addressing
/// [`crate::RuleCalibration`] uses for a whole rule: an approval is a team's own record that
/// a human or agent accepted this phase's findings despite its threshold, not a per-finding
/// override the way [`crate::Suppression`] and [`crate::BaselineDebt`] both are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseApproval
{
    /// The [`crate::GatePhase::name`] this approval covers.
    pub phase: String,
    /// Why this phase was approved despite exceeding its threshold -- required, the same
    /// "never a silent override" discipline [`crate::Suppression::rationale`] and
    /// [`crate::BaselineDebt::rationale`] both already hold.
    pub rationale: String,
}
