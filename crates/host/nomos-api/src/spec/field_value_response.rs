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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_model::Origin;

    #[test]
    fn Test_From_Should_Copy_Every_Field_And_Map_The_Nested_Origin()
    {
        let value = FieldValue { field: "title".to_owned(), value: "t".to_owned(), origin: Origin::Submitted };

        let response = FieldValueResponse::From(value.clone());

        assert_eq!(response.field, value.field);
        assert_eq!(response.value, value.value);
        assert!(matches!(response.origin, OriginResponse::Submitted));
    }
}
