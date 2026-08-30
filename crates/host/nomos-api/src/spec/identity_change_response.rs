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
