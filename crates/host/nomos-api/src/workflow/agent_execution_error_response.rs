//! [`AgentExecutionErrorResponse`], carried only by
//! [`super::DispatchErrorResponse::ClaudeCode`].

use serde::Serialize;

/// A serializable twin of [`nomos_agent_executor_claude_code::AgentExecutionError`].
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum AgentExecutionErrorResponse
{
    Unavailable
    {
        reason: String,
    },
    Unparseable
    {
        reason: String,
    },
    /// A path the envelope forbade changing was changed. Carries that path, not a
    /// `reason`: the wire shape says which field of the envelope was violated rather
    /// than flattening it into prose a caller would have to parse back.
    ProhibitedChange
    {
        path: String,
    },
    /// The envelope declared capabilities no tool grant exists for, so nothing ran.
    UnsupportedTools
    {
        capabilities: String,
    },
    /// Paths to protect were declared against a root that does not say which tree.
    UnresolvableRoot
    {
        root: String,
    },
}

impl AgentExecutionErrorResponse
{
    pub(crate) fn From(error: nomos_agent_executor_claude_code::AgentExecutionError) -> Self
    {
        return match error
        {
            nomos_agent_executor_claude_code::AgentExecutionError::Unavailable(reason) => Self::Unavailable { reason },
            nomos_agent_executor_claude_code::AgentExecutionError::Unparseable(reason) => Self::Unparseable { reason },
            nomos_agent_executor_claude_code::AgentExecutionError::ProhibitedChange(path) => Self::ProhibitedChange { path },
            nomos_agent_executor_claude_code::AgentExecutionError::UnsupportedTools(capabilities) => Self::UnsupportedTools { capabilities },
            nomos_agent_executor_claude_code::AgentExecutionError::UnresolvableRoot(root) => Self::UnresolvableRoot { root },
        };
    }
}
