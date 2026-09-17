//! [`AgentDispatchResponse`], what [`super::Handle_Agent_Execute`] and
//! [`super::Handle_Agent_Judge_Role`] answer.

use serde::Serialize;

/// A serializable twin of [`nomos_agent_orchestration::AgentDispatchOutcome`].
///
/// `ClaudeCode`'s `assumptions`/`unresolved_questions` are
/// [`nomos_agent_contracts::WorkResult`]'s own two fields this executor can honestly
/// populate -- `OD-EXECUTOR-008`'s decision. `plan`, `claims`, `tests` and
/// `requested_verification` are not projected here because they are always structurally
/// absent for this executor, never because a wire caller could not use them if they existed.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "backend")]
pub enum AgentDispatchResponse
{
    ClaudeCode
    {
        assumptions: Vec<String>, unresolved_questions: Vec<String>, denied_tool_uses: Vec<String>,
        is_error: bool, cost_usd: f64, duration_ms: u64
    },
    Ollama
    {
        response: String
    },
    Unavailable
    {
        reason: String
    },
}
