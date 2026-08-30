//! [`SubmissionResponse`], carried only by [`super::submit_response::SubmitResponse::Accepted`].

use nomos_spec_model::Submission;
use serde::Serialize;

use super::{DecisionGapResponse, FieldValueResponse, SubmissionKindResponse, SubmissionStateResponse};

/// A serializable twin of [`nomos_spec_model::Submission`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct SubmissionResponse
{
    /// Identity, assigned by the submitter rather than by the store.
    pub id: String,
    pub kind: SubmissionKindResponse,
    pub form_contract_version: u32,
    pub state: SubmissionStateResponse,
    pub submitted_by: String,
    pub submitted_through: String,
    pub values: Vec<FieldValueResponse>,
    pub gaps: Vec<DecisionGapResponse>,
}

impl SubmissionResponse
{
    pub(crate) fn From(submission: Submission) -> Self
    {
        return Self {
            id: submission.id,
            kind: SubmissionKindResponse::From(submission.kind),
            form_contract_version: submission.form_contract_version,
            state: SubmissionStateResponse::From(submission.state),
            submitted_by: submission.submitted_by,
            submitted_through: submission.submitted_through,
            values: submission.values.into_iter().map(FieldValueResponse::From).collect(),
            gaps: submission.gaps.into_iter().map(DecisionGapResponse::From).collect(),
        };
    }
}
