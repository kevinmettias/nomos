//! [`OllamaExecutionErrorResponse`], carried only by
//! [`super::DispatchErrorResponse::Ollama`].

use serde::Serialize;

/// A serializable twin of [`nomos_model_backend_ollama::AgentExecutionError`].
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum OllamaExecutionErrorResponse
{
    Unavailable
    {
        reason: String,
    },
    /// The envelope declared capabilities no tool grant exists for, so nothing ran. Spelled
    /// the same as its sibling's over the wire, because a caller meets one refusal for one
    /// reason and should not have to learn which backend phrased it.
    UnsupportedTools
    {
        capabilities: String,
    },
}

impl OllamaExecutionErrorResponse
{
    pub(crate) fn From(error: nomos_model_backend_ollama::AgentExecutionError) -> Self
    {
        return match error
        {
            nomos_model_backend_ollama::AgentExecutionError::Unavailable(reason) => Self::Unavailable { reason },
            nomos_model_backend_ollama::AgentExecutionError::UnsupportedTools(capabilities) => Self::UnsupportedTools { capabilities },
        };
    }
}
