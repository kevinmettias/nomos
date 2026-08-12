//! What a commit did.

/// What a commit did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitReport
{
    pub node_id: String,
    pub path: String,
    pub blocks: usize,
    pub blocks_removed: usize,
    pub relations_added: usize,
    pub relations_removed: usize,
    pub renamed: bool,
}
