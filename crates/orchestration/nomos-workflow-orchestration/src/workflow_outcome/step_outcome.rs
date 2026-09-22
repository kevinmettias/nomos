//! What one dispatched step reported.

/// What one dispatched step reported.
///
/// The agent arm carries `nomos_agent_orchestration::AgentDispatchOutcome` whole rather
/// than a per-backend variant of its own. That type keeps the two answers apart -- an
/// executor's execution carries a denial list, an error flag, a spend and a duration; a
/// model backend's answer carries a response and none of those -- so nothing is flattened
/// by carrying it here, and this crate names no adapter to do it.
#[derive(Clone, Debug, PartialEq)]
pub enum StepOutcome
{
    /// What the target the step's profile resolved to reported, or why none was selected.
    Agent(nomos_agent_orchestration::AgentDispatchOutcome),
    /// What `nomos-check-orchestration::Run` reported.
    Check(nomos_check_orchestration::CheckOutcome),
    /// What `nomos-correction-orchestration::Run_Correction` reported.
    Correction(nomos_correction_orchestration::CorrectionOutcome),
    /// What `nomos-gate-orchestration::Run_Gate` reported, for a run that did not fail --
    /// see [`crate::DispatchError::Gate`] for the one that did.
    Gate(nomos_gate_orchestration::GateRunResult),
}
