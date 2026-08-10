//! What `nomos spec commit` was asked for.

use std::path::PathBuf;
use crate::spec::edit_request::EditRequest;
/// An edit, and the tree its record's own path is written under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitRequest
{
    /// What is being committed.
    pub edit: EditRequest,
    /// The tree the record's own path is written under. A record's path is repository
    /// relative, so a commit has to be told which tree it means; `.` is the default rather
    /// than the only option, so a test does not have to write into the tree it is testing.
    pub into: PathBuf,
}
