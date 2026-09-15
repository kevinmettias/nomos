//! [`OllamaExecutionOutcomeResponse`], carried only by
//! [`super::StepOutcomeResponse::Ollama`].

use serde::Serialize;

/// A serializable twin of [`nomos_model_backend_ollama::AgentExecutionOutcome`].
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OllamaExecutionOutcomeResponse
{
    pub response: String,
}

impl OllamaExecutionOutcomeResponse
{
    pub(crate) fn From(outcome: nomos_model_backend_ollama::AgentExecutionOutcome) -> Self
    {
        return Self { response: outcome.response };
    }
}
