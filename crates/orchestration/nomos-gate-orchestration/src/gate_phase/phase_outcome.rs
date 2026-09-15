//! One phase's own outcome, after [`crate::Evaluated_Phases`] judged it.

use nomos_contracts::Finding;

use super::PhaseDisposition;

/// One phase's own outcome, after [`crate::Evaluated_Phases`] judged it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseOutcome
{
    /// The [`crate::GatePhase::name`] this outcome answers for.
    pub name: String,
    /// Whether this phase was judged at all -- `false` for every phase after the first one
    /// that failed unapproved. This crate already computed every finding in one pass
    /// (`crate::gate_environment::Judged_Sources`), so nothing here re-runs a rule; what
    /// changes is whether a phase's own verdict is judged and reported at all, or left
    /// unjudged because an earlier phase in the declared order already stopped the run.
    pub ran: bool,
    /// This phase's own blocking findings -- carried even when [`Self::approved`] is `true`,
    /// the same "never silently drop a finding" discipline `crate::GateFindings`'s own doc
    /// already states for calibration, suppression and baseline.
    pub blocking_findings: Vec<Finding>,
    /// Whether a [`crate::PhaseApproval`] matching this phase's name let it pass despite
    /// exceeding its threshold. `false` when nothing needed approving.
    pub approved: bool,
    /// This phase's own verdict.
    pub disposition: PhaseDisposition,
}
