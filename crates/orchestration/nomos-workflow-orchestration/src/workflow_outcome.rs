//! What running a `WorkflowStepPlan` sequence produced.

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
}

/// Why a step's dispatch failed — naming each backend's own error type directly, the
/// same reason [`StepOutcome`] does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchError
{
    /// Why `nomos-agent-executor-claude-code::Execute_Task` could not answer.
    ClaudeCode(nomos_agent_executor_claude_code::AgentExecutionError),
    /// Why `nomos-model-backend-ollama::Execute_Task` could not answer.
    Ollama(nomos_model_backend_ollama::AgentExecutionError),
}

/// What one [`crate::Run`] produced.
#[derive(Clone, Debug, PartialEq)]
pub enum WorkflowOutcome
{
    /// Every step in the plan ran and dispatched successfully, in order. Vacuously true
    /// of an empty plan.
    Completed
    {
        /// Every step's real outcome, in the order the plan declared them.
        completed: Vec<StepOutcome>,
    },
    /// The step at `index` declared itself incoherent and was never dispatched.
    Refused
    {
        /// Every real outcome from the steps that ran before `index`, in order.
        completed: Vec<StepOutcome>,
        /// The position in the plan of the step that was refused.
        index: usize,
    },
    /// The step at `index` was dispatched and its dispatch failed.
    Failed
    {
        /// Every real outcome from the steps that ran before `index`, in order.
        completed: Vec<StepOutcome>,
        /// The position in the plan of the step whose dispatch failed.
        index: usize,
        /// Why the dispatch failed.
        error: DispatchError,
    },
}
