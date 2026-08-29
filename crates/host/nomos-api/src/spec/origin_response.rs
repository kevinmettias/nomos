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
