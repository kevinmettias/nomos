//! What one dispatched step reported.

/// What one dispatched step reported — naming each backend's own outcome type directly,
/// the same no-shared-trait shape [`crate::Body`] already uses, because the two crates'
/// outcome shapes are not interchangeable: `nomos-agent-executor-claude-code`'s carries
/// `denied_tool_uses`, `is_error`, `cost` and `duration_ms` that
/// `nomos-model-backend-ollama`'s honestly does not have.
#[derive(Clone, Debug, PartialEq)]
pub enum StepOutcome
{
    /// What `nomos-agent-executor-claude-code::Execute_Task` reported.
    /// What `nomos-model-backend-ollama::Execute_Task` reported.
    /// What the backend the step's profile resolved to reported, or why none was selected.
    Agent(nomos_agent_orchestration::AgentDispatchOutcome),
    /// What `nomos-check-orchestration::Run` reported.
    Check(nomos_check_orchestration::CheckOutcome),
    /// What `nomos-correction-orchestration::Run_Correction` reported.
    Correction(nomos_correction_orchestration::CorrectionOutcome),
    /// What `nomos-gate-orchestration::Run_Gate` reported, for a run that did not fail --
    /// see [`crate::DispatchError::Gate`] for the one that did.
    Gate(nomos_gate_orchestration::GateRunResult),
}
