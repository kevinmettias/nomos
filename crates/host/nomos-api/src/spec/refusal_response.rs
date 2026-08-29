//! [`RefusalResponse`], carried only by [`super::spec_submit_response::SpecSubmitResponse::Refused`].

use nomos_spec_model::Refusal;
use serde::Serialize;

use super::FailureResponse;

/// A serializable twin of [`nomos_spec_model::Refusal`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
pub struct RefusalResponse
{
    /// Which submission was refused.
    pub submission: String,
    /// Every rule it failed.
    pub failures: Vec<FailureResponse>,
}

impl RefusalResponse
{
    pub(crate) fn From(refusal: Refusal) -> Self
    {
        return Self {
            submission: refusal.submission,
            failures: refusal.failures.into_iter().map(FailureResponse::From).collect(),
        };
    }
}
