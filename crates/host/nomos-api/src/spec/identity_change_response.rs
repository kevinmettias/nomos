//! [`IdentityChangeResponse`], shared by [`super::preview_response::PreviewResponse::Previewed`]
//! and [`super::committed_preview_response::CommittedPreviewResponse`].

use nomos_spec_store::IdentityChange;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_store::IdentityChange`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct IdentityChangeResponse
{
    pub field: String,
    pub before: String,
    pub after: String,
}

impl IdentityChangeResponse
{
    pub(crate) fn From(change: IdentityChange) -> Self
    {
        return Self { field: change.field, before: change.before, after: change.after };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Copy_Every_Field_Of_The_Domain_Identity_Change()
    {
        let change = IdentityChange {
            field: "id".to_owned(),
            before: "D-131".to_owned(),
            after: "D-132".to_owned(),
        };

        let response = IdentityChangeResponse::From(change.clone());

        assert_eq!(response.field, change.field);
        assert_eq!(response.before, change.before);
        assert_eq!(response.after, change.after);
    }
}
