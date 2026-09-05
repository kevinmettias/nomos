//! What one dispatched step reported.

/// What one dispatched step reported — naming each backend's own outcome type directly,
/// the same no-shared-trait shape [`crate::Body`] already uses, because the two crates'
/// outcome shapes are not interchangeable: `nomos-agent-executor-claude-code`'s carries
/// `denied_tool_uses`, `is_error`, `cost_usd` and `duration_ms` that
/// `nomos-model-backend-ollama`'s honestly does not have.
#[derive(Clone, Debug, PartialEq)]
pub enum StepOutcome
{
    /// What `nomos-agent-executor-claude-code::Execute_Task` reported.
    ClaudeCode(nomos_agent_executor_claude_code::AgentExecutionOutcome),
    /// What `nomos-model-backend-ollama::Execute_Task` reported.
    Ollama(nomos_model_backend_ollama::AgentExecutionOutcome),
    /// What `nomos-check-orchestration::Run` reported.
    Check(nomos_check_orchestration::CheckOutcome),
}
