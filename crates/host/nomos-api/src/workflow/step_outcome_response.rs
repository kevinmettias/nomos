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
    ClaudeCode(AgentExecutionOutcomeResponse),
    Ollama(OllamaExecutionOutcomeResponse),
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
            StepOutcome::ClaudeCode(outcome) => Self::ClaudeCode(AgentExecutionOutcomeResponse::From(outcome)),
            StepOutcome::Ollama(outcome) => Self::Ollama(OllamaExecutionOutcomeResponse::From(outcome)),
            StepOutcome::Check(outcome) => Self::Check(check::CheckResponse::From(outcome)),
            StepOutcome::Correction(outcome) => Self::Correction(correction::CorrectionResponse::From(outcome)),
            StepOutcome::Gate(result) => Self::Gate(GateRunResponse::From(result)),
        };
    }
}
