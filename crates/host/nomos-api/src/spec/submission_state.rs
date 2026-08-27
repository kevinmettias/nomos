//! [`SubmissionStateResponse`], carried only by [`super::submission::SubmissionResponse`].

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
