//! [`StepOutcomeResponse`], carried inside every [`super::WorkflowRunResponse`]'s own
//! `completed` list.

use crate::response::GateRunResponse;
use crate::{check, correction};
use nomos_workflow_orchestration::StepOutcome;
use serde::Serialize;

use super::{AgentExecutionOutcomeResponse, OllamaExecutionOutcomeResponse};

/// A serializable twin of [`nomos_workflow_orchestration::StepOutcome`].
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "backend")]
pub enum StepOutcomeResponse
{
    /// What the backend this step resolved to reported, or why none was selected.
    Agent(crate::agent::AgentDispatchResponse),
    Check(check::CheckResponse),
    Correction(correction::CorrectionResponse),
    Gate(GateRunResponse),
}

impl StepOutcomeResponse
{
    pub(crate) fn From(outcome: StepOutcome) -> Self
    {
        return match outcome
        {
            StepOutcome::Agent(outcome) => Self::Agent(crate::agent::AgentDispatchResponse::From(outcome)),
            StepOutcome::Check(outcome) => Self::Check(check::CheckResponse::From(outcome)),
            StepOutcome::Correction(outcome) => Self::Correction(correction::CorrectionResponse::From(outcome)),
            StepOutcome::Gate(result) => Self::Gate(GateRunResponse::From(result)),
        };
    }
}
