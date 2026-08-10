//! One file as a snapshot holds it.

use nomos_contracts::Digest128;
/// One file in the workspace.
///
/// A path and a content address. Not the content: a snapshot of a large tree that carried
/// every byte would be a copy of the tree, and the store already holds content by address.
/// What the snapshot has to be able to say without the tree is *which* content, and a
/// digest says that.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Member
{
    /// Workspace-relative, normalized. Never absolute, which is the whole of portability.
    pub path: String,
    pub content: Digest128,
}
