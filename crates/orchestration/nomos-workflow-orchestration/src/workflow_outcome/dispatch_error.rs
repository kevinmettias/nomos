//! Why a step's dispatch failed.

/// Why a step's dispatch failed.
///
/// `Gate` carries the whole [`nomos_gate_orchestration::GateRunResult`], not a narrower
/// error type: that crate has no separate error type for a failing run, because
/// `GateRunResult` itself is both the success and the failure shape, unified. A caller
/// reading `DispatchError::Gate` sees exactly the disposition, findings and check facts
/// it would have seen from a `StepOutcome::Gate` that merely completed -- what changes is
/// only that a failing one stops the workflow rather than letting it continue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchError
{
    /// The target this step's profile resolved to could not be started, or did not answer.
    ///
    /// A dispatch error rather than a step outcome, because it stops the run: that is what a
    /// failing agent step did before the two per-backend variants were collapsed, and
    /// collapsing them was about which backend answers, not about whether a failure ends the
    /// workflow.
    AgentUnavailable(String),
    /// No backend was selected, so nothing was dispatched at all.
    ///
    /// Kept apart from [`Self::AgentUnavailable`] for the reason
    /// `nomos_agent_orchestration::BackendAbsence` exists: a backend that failed and a
    /// declaration that offered nothing have different remedies.
    AgentNotSelected(nomos_agent_orchestration::BackendAbsence),
    Gate(nomos_gate_orchestration::GateRunResult),
}
