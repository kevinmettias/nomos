//! [`SubmissionStateResponse`], carried only by [`super::submission_response::SubmissionResponse`].

use nomos_spec_model::SubmissionState;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_model::SubmissionState`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionStateResponse
{
    Draft,
    Accepted,
}

impl SubmissionStateResponse
{
    pub(crate) fn From(state: SubmissionState) -> Self
    {
        return match state
        {
            SubmissionState::Draft => Self::Draft,
            SubmissionState::Accepted => Self::Accepted,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Map_Every_Domain_Variant_To_Its_Own_Response_Variant()
    {
        assert!(matches!(SubmissionStateResponse::From(SubmissionState::Draft), SubmissionStateResponse::Draft));
        assert!(matches!(
            SubmissionStateResponse::From(SubmissionState::Accepted),
            SubmissionStateResponse::Accepted
        ));
    }
}
