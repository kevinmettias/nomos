//! [`NodeSummaryResponse`], carried only by [`super::record_response::RecordResponse::NotFound`].

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Copy_Every_Field_Of_The_Domain_Node_Summary()
    {
        let node = NodeSummary {
            node_id: "D-132".to_owned(),
            kind: "record".to_owned(),
            authority: "governing".to_owned(),
            representation: "markdown".to_owned(),
            title: "Example Record".to_owned(),
        };

        let response = NodeSummaryResponse::From(node.clone());

        assert_eq!(response.node_id, node.node_id);
        assert_eq!(response.kind, node.kind);
        assert_eq!(response.authority, node.authority);
        assert_eq!(response.representation, node.representation);
        assert_eq!(response.title, node.title);
    }
}
