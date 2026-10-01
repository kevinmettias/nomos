//! [`OriginResponse`], carried only by [`super::field_value_response::FieldValueResponse`].

use nomos_spec_model::Origin;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_model::Origin`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OriginResponse
{
    Submitted,
    Clarified,
    Inferred,
    Decided,
}

impl OriginResponse
{
    pub(crate) fn From(origin: Origin) -> Self
    {
        return match origin
        {
            Origin::Submitted => Self::Submitted,
            Origin::Clarified => Self::Clarified,
            Origin::Inferred => Self::Inferred,
            Origin::Decided => Self::Decided,
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
        assert!(matches!(OriginResponse::From(Origin::Submitted), OriginResponse::Submitted));
        assert!(matches!(OriginResponse::From(Origin::Clarified), OriginResponse::Clarified));
        assert!(matches!(OriginResponse::From(Origin::Inferred), OriginResponse::Inferred));
        assert!(matches!(OriginResponse::From(Origin::Decided), OriginResponse::Decided));
    }
}
