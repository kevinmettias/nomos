//! [`WorkflowRunResponse`], what [`super::Handle_Workflow_Run`] answers.

use super::{DispatchErrorResponse, StepOutcomeResponse};
use serde::Serialize;

/// A serializable twin of [`nomos_workflow_orchestration::WorkflowOutcome`].
///
/// A twin rather than a re-export because the type it mirrors does not derive `Serialize`,
/// for the reason [`crate::response`]'s own doc gives. `UnreadableRoot` is not one of
/// [`nomos_workflow_orchestration::WorkflowOutcome`]'s own three variants: it is this
/// handler's own composition-root answer for a named root that is not a directory, the
/// identical case `nomos-cli`'s own `workflow.rs` reports as a bare exit code, outside
/// `Rendered`'s own match, rather than inventing a fourth case inside `WorkflowOutcome`
/// itself for a question that function never asks.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum WorkflowRunResponse
{
    UnreadableRoot
    {
        reason: String,
    },
    Completed
    {
        completed: Vec<StepOutcomeResponse>,
    },
    Refused
    {
        completed: Vec<StepOutcomeResponse>, index: usize,
    },
    Failed
    {
        completed: Vec<StepOutcomeResponse>, index: usize, error: DispatchErrorResponse,
    },
}
