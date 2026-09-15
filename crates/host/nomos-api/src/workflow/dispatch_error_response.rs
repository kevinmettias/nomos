//! [`DispatchErrorResponse`], carried only by
//! [`super::WorkflowRunResponse::Failed`].

use crate::response::GateRunResponse;
use nomos_workflow_orchestration::DispatchError;
use serde::Serialize;

use super::{AgentExecutionErrorResponse, OllamaExecutionErrorResponse};

/// A serializable twin of [`nomos_workflow_orchestration::DispatchError`].
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "backend")]
pub enum DispatchErrorResponse
{
    ClaudeCode(AgentExecutionErrorResponse),
    Ollama(OllamaExecutionErrorResponse),
    Gate(GateRunResponse),
}

impl DispatchErrorResponse
{
    pub(crate) fn From(error: DispatchError) -> Self
    {
        return match error
        {
            DispatchError::ClaudeCode(error) => Self::ClaudeCode(AgentExecutionErrorResponse::From(error)),
            DispatchError::Ollama(error) => Self::Ollama(OllamaExecutionErrorResponse::From(error)),
            DispatchError::Gate(result) => Self::Gate(GateRunResponse::From(result)),
        };
    }
}
