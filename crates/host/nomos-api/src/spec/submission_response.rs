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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_model::{FieldValue, Origin, SubmissionKind, SubmissionState};

    #[test]
    fn Test_From_Should_Copy_Every_Field_And_Map_Every_Nested_Value()
    {
        let submission = A_Submission();

        let response = SubmissionResponse::From(submission.clone());

        assert_eq!(response.id, submission.id);
        assert!(matches!(response.kind, SubmissionKindResponse::FeatureRequest));
        assert_eq!(response.form_contract_version, submission.form_contract_version);
        assert!(matches!(response.state, SubmissionStateResponse::Draft));
        assert_eq!(response.submitted_by, submission.submitted_by);
        assert_eq!(response.submitted_through, submission.submitted_through);
        assert_eq!(response.values.len(), 1);
        assert_eq!(
            response.values.first().expect("asserted above to contain exactly one value").field,
            "title"
        );
        assert!(response.gaps.is_empty());
    }

    /// A real submission holding one value and no gap, so a mapping that dropped either a
    /// field or a nested value would show.
    fn A_Submission() -> Submission
    {
        return Submission {
            id: "FR-API-001".to_owned(),
            kind: SubmissionKind::FeatureRequest,
            form_contract_version: 1,
            state: SubmissionState::Draft,
            submitted_by: "kevin".to_owned(),
            submitted_through: "nomos-api-test".to_owned(),
            values: vec![FieldValue { field: "title".to_owned(), value: "t".to_owned(), origin: Origin::Submitted }],
            gaps: Vec::new(),
        };
    }
}
