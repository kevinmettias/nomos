//! [`DispatchErrorResponse`], carried only by
//! [`super::WorkflowRunResponse::Failed`].

use crate::response::GateRunResponse;
use nomos_workflow_orchestration::DispatchError;
use serde::Serialize;

/// A serializable twin of [`nomos_workflow_orchestration::DispatchError`].
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "backend")]
pub enum DispatchErrorResponse
{
    /// The backend the step resolved to could not be started, or did not answer.
    AgentUnavailable
    {
        reason: String
    },
    /// No backend was selected, so nothing was dispatched.
    AgentNotSelected
    {
        absence: String
    },
    Gate(GateRunResponse),
}

impl DispatchErrorResponse
{
    pub(crate) fn From(error: DispatchError) -> Self
    {
        return match error
        {

            DispatchError::AgentUnavailable(reason) => Self::AgentUnavailable { reason },
            DispatchError::AgentNotSelected(absence) => Self::AgentNotSelected { absence: format!("{absence:?}") },
            DispatchError::Gate(result) => Self::Gate(GateRunResponse::From(result)),
        };
    }
}
