//! [`RecordRelationResponse`], shared by [`super::spec_preview_response::SpecPreviewResponse::Previewed`]
//! and [`super::committed_preview_response::CommittedPreviewResponse`].

use nomos_spec_model::RecordRelation;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_model::RecordRelation`], which derives `Deserialize`
/// but not `Serialize`.
#[derive(Debug, Serialize)]
pub struct RecordRelationResponse
{
    pub target: String,
    pub relation: String,
}

impl RecordRelationResponse
{
    pub(crate) fn From(relation: RecordRelation) -> Self
    {
        return Self { target: relation.target, relation: relation.relation };
    }
}
