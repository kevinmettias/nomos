//! A node as the graph holds it, with no content behind it.

/// A node as the graph holds it, with no content behind it.
///
/// Separate from [`RecordSource`] because a node with no source document is a real and
/// common answer — the catalog mints thousands of them — and reporting that as "not
/// found" would tell a reader the identifier is unknown when the store knows it well.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeSummary
{
    pub node_id: String,
    pub kind: String,
    pub authority: String,
    pub representation: String,
    pub title: String,
}
