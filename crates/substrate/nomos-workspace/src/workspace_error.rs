//! Every way the workspace refuses a change.
//!
//! # Why this file sits at the crate root rather than inside `workspace/`
//!
//! It was `workspace/error.rs`, declaring `Error`, and `workspace.rs` published it as
//! `pub use error::Error as WorkspaceError;`. The same three rules that put
//! [`crate::WorkspaceChangeSet`] here apply one file along, and that file's header states
//! the whole of it: the public name forces the file to be `workspace_error`,
//! `workspace/workspace_error.rs` would repeat its parent folder, and keeping `Error` would
//! need the alias.

use nomos_store::StoreError;

/// A refusal raised while applying a change set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceError
{
    /// A change set with nothing in it.
    ///
    /// Refused rather than accepted as `Unchanged`, because they are different mistakes. A
    /// set whose changes all turned out to be redundant is a submitter who did not know;
    /// an empty set is a submitter who built one and never put anything in it, and that is
    /// almost always a loop that iterated zero times.
    Vacuous,
    /// A path that does not name a workspace-relative file.
    Unnamed
    {
        path: String,
        reason: String,
    },
    /// The same path changed twice in one set.
    ///
    /// Refused rather than last-wins, because a set that says a path is both present and
    /// absent has no correct interpretation and picking one silently would make the
    /// workspace's state depend on the order a caller happened to push changes.
    Conflicting
    {
        path: String,
    },
    Store(StoreError),
}

impl core::fmt::Display for WorkspaceError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Vacuous => write!(
                formatter,
                "a change set with no changes in it. This is refused rather than ignored \
                 because it is almost always a loop that iterated zero times"
            ),
            Self::Unnamed { path, reason } => {
                write!(formatter, "`{path}` cannot name a workspace member: {reason}")
            }
            Self::Conflicting { path } => write!(
                formatter,
                "`{path}` is changed twice in one set. There is no correct reading of a \
                 path that is both present and absent, and choosing one would make the \
                 workspace depend on the order a caller pushed changes"
            ),
            Self::Store(error) => error.fmt(formatter),
        };
    }
}

impl std::error::Error for WorkspaceError
{}

impl From<StoreError> for WorkspaceError
{
    fn from(error: StoreError) -> Self
    {
        return Self::Store(error);
    }
}
