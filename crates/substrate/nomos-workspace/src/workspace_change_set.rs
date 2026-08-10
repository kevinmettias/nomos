//! Every change observed in one pass, as one thing to reason about.

use crate::change::Change;
use crate::change_source::ChangeSource;
/// A batch of changes from one source, applied as one step.
///
/// A batch rather than a change, because a checkout that moved four hundred files is one
/// event. Applying them one at a time would produce four hundred generations, and every
/// intermediate one would describe a tree that never existed — a half-applied checkout is
/// not a state anybody should be able to ask questions about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceChangeSet
{
    source: ChangeSource,
    changes: Vec<Change>,
}

impl WorkspaceChangeSet
{
    #[must_use]
    pub const fn From(source: ChangeSource) -> Self
    {
        return Self {
            source,
            changes: Vec::new(),
        };
    }

    #[must_use]
    pub fn Present(mut self, path: impl Into<String>, content: impl Into<String>) -> Self
    {
        self.changes.push(Change::Present {
            path: path.into(),
            content: content.into(),
        });

        return self;
    }

    #[must_use]
    pub fn Absent(mut self, path: impl Into<String>) -> Self
    {
        self.changes.push(Change::Absent { path: path.into() });

        return self;
    }

    #[must_use]
    pub const fn Source(&self) -> ChangeSource
    {
        return self.source;
    }

    #[must_use]
    pub fn Changes(&self) -> &[Change]
    {
        return &self.changes;
    }

    #[must_use]
    pub fn Is_Empty(&self) -> bool
    {
        return self.changes.is_empty();
    }
}
