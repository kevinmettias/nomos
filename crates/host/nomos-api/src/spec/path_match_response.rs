//! [`PathMatchResponse`], carried only by [`super::table_response::TableResponse`].

use nomos_spec_store::PathMatch;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_store::PathMatch`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathMatchResponse
{
    /// The path was given in full.
    Exact,
    /// The last segment of the path was given.
    FileName,
    /// The text appears somewhere in the path.
    Fragment,
}

impl PathMatchResponse
{
    pub(crate) fn From(tier: PathMatch) -> Self
    {
        return match tier
        {
            PathMatch::Exact => Self::Exact,
            PathMatch::FileName => Self::FileName,
            PathMatch::Fragment => Self::Fragment,
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
        assert!(matches!(PathMatchResponse::From(PathMatch::Exact), PathMatchResponse::Exact));
        assert!(matches!(PathMatchResponse::From(PathMatch::FileName), PathMatchResponse::FileName));
        assert!(matches!(PathMatchResponse::From(PathMatch::Fragment), PathMatchResponse::Fragment));
    }
}
