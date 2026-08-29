//! [`NodeSummaryResponse`], carried only by [`super::spec_record_response::SpecRecordResponse::NotFound`].

use nomos_spec_store::NodeSummary;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_store::NodeSummary`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct NodeSummaryResponse
{
    pub node_id: String,
    pub kind: String,
    pub authority: String,
    pub representation: String,
    pub title: String,
}

impl NodeSummaryResponse
{
    pub(crate) fn From(node: NodeSummary) -> Self
    {
        return Self {
            node_id: node.node_id,
            kind: node.kind,
            authority: node.authority,
            representation: node.representation,
            title: node.title,
        };
    }
}
