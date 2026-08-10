//! What `nomos spec edit` was asked for.

use std::path::PathBuf;
/// The edit itself, without saying whether it will be committed.
///
/// `preview` and `commit` take the same edit and differ only in what they do with it,
/// which is what makes "the preview and then the commit, in that order" expressible at
/// all: `commit` holds one of these and runs the preview from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditRequest
{
    /// The node identifier.
    pub id: String,
    /// The edited markdown.
    pub from: PathBuf,
    /// The path the record should move to. A rename is an ordinary edit.
    pub rename: Option<String>,
}
