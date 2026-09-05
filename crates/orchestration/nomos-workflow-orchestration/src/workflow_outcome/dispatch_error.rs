//! Why a step's dispatch failed.

/// Why a step's dispatch failed — naming each backend's own error type directly, the
/// same reason [`super::StepOutcome`] does.
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
    /// Why `nomos-agent-executor-claude-code::Execute_Task` could not answer.
    ClaudeCode(nomos_agent_executor_claude_code::AgentExecutionError),
    /// Why `nomos-model-backend-ollama::Execute_Task` could not answer.
    Ollama(nomos_model_backend_ollama::AgentExecutionError),
    /// A `Body::Gate` step whose own `GateRunOutcome` was `Failed`.
    Gate(nomos_gate_orchestration::GateRunResult),
}
