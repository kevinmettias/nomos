//! [`FieldValueResponse`], carried only by [`super::submission_response::SubmissionResponse`].

use nomos_spec_model::FieldValue;
use serde::Serialize;

use super::OriginResponse;

/// A serializable twin of [`nomos_spec_model::FieldValue`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct FieldValueResponse
{
    pub field: String,
    pub value: String,
    pub origin: OriginResponse,
}

impl FieldValueResponse
{
    pub(crate) fn From(value: FieldValue) -> Self
    {
        return Self { field: value.field, value: value.value, origin: OriginResponse::From(value.origin) };
    }
}
