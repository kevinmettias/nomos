//! [`AgentExecutionOutcomeResponse`], carried only by
//! [`super::StepOutcomeResponse::ClaudeCode`].

use serde::Serialize;

/// A serializable twin of [`nomos_agent_executor_claude_code::AgentExecutionOutcome`].
///
/// `assumptions`/`unresolved_questions` are [`nomos_agent_contracts::WorkResult`]'s own two
/// fields this executor can honestly populate -- `OD-EXECUTOR-008`'s decision. `plan`,
/// `claims`, `tests` and `requested_verification` are not projected here because they are
/// always structurally absent for this executor, never because a wire caller could not use
/// them if they existed.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentExecutionOutcomeResponse
{
    pub assumptions: Vec<String>,
    pub unresolved_questions: Vec<String>,
    pub denied_tool_uses: Vec<String>,
    pub is_error: bool,
    pub cost_usd: f64,
    pub duration_ms: u64,
}

impl AgentExecutionOutcomeResponse
{
    pub(crate) fn From(outcome: nomos_agent_executor_claude_code::AgentExecutionOutcome) -> Self
    {
        return Self {
            assumptions: outcome.result.assumptions,
            unresolved_questions: outcome.result.unresolved_questions,
            denied_tool_uses: outcome.denied_tool_uses,
            is_error: outcome.is_error,
            cost_usd: crate::agent::Dollars_Of(outcome.cost),
            duration_ms: outcome.duration_ms,
        };
    }
}
