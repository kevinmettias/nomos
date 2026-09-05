//! Why a step's dispatch failed.

/// Why a step's dispatch failed — naming each backend's own error type directly, the
/// same reason [`super::StepOutcome`] does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchError
{
    /// Why `nomos-agent-executor-claude-code::Execute_Task` could not answer.
    ClaudeCode(nomos_agent_executor_claude_code::AgentExecutionError),
    /// Why `nomos-model-backend-ollama::Execute_Task` could not answer.
    Ollama(nomos_model_backend_ollama::AgentExecutionError),
}
