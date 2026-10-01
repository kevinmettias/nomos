//! Every change observed in one pass, as one thing to reason about.
//!
//! # Why this file sits at the crate root rather than inside `workspace/`
//!
//! It was `workspace/change_set.rs`, declaring `ChangeSet`, and `workspace.rs` published it
//! as `pub use change_set::ChangeSet as WorkspaceChangeSet;`. Three rules pull against that
//! shape, and moving the file here is the only arrangement that satisfies all three at once:
//!
//! - `file-name-matches-declared-type` requires a file to be named after the public type it
//!   declares, so a type called `WorkspaceChangeSet` forces the file to be
//!   `workspace_change_set`.
//! - `check-tree-legibility` forbids a file repeating its parent folder, so
//!   `workspace/workspace_change_set.rs` is out.
//! - `check-facade-surface` counts `pub use x::Y as Z;` a second public name for one item, so
//!   keeping `ChangeSet` and qualifying it at the facade is out too.
//!
//! The public path is unchanged: `lib.rs` re-exports this, so
//! `nomos_workspace::WorkspaceChangeSet` is what it always was. The crate previously
//! published this one type under two names, `ChangeSet` and `WorkspaceChangeSet`; nothing
//! outside the crate ever named the first, and a second public name is the whole of what
//! `check-facade-surface` reports, so it is gone rather than aliased.

use crate::Change;
use crate::ChangeSource;

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Begin_With_Nothing_Recorded_Yet()
    {
        let set = WorkspaceChangeSet::From(ChangeSource::GitCheckout);

        assert_eq!(set.Source(), ChangeSource::GitCheckout);
        assert!(set.Is_Empty());
    }

    #[test]
    fn Test_Present_Should_Append_One_Change_With_Its_Content()
    {
        let set = WorkspaceChangeSet::From(ChangeSource::IdeEdit).Present("src/a.rs", "fn a() {}");

        assert_eq!(set.Changes().len(), 1);
        assert_eq!(
            set.Changes().first().expect("Present just appended one change"),
            &Change::Present {
                path: "src/a.rs".to_owned(),
                content: "fn a() {}".to_owned(),
            }
        );
    }

    #[test]
    fn Test_Absent_Should_Append_A_Change_With_No_Content()
    {
        let set = WorkspaceChangeSet::From(ChangeSource::AgentEdit).Absent("src/gone.rs");

        assert_eq!(set.Changes().len(), 1);
        assert_eq!(
            set.Changes().first().expect("Absent just appended one change"),
            &Change::Absent {
                path: "src/gone.rs".to_owned(),
            }
        );
    }

    #[test]
    fn Test_Source_Should_Report_What_Was_Given_At_Construction()
    {
        let set = WorkspaceChangeSet::From(ChangeSource::CodeGenerator);

        assert_eq!(set.Source(), ChangeSource::CodeGenerator);
    }

    #[test]
    fn Test_Changes_Should_List_Both_Kinds_In_The_Order_Added()
    {
        let set = WorkspaceChangeSet::From(ChangeSource::Correction)
            .Present("a.rs", "one")
            .Absent("b.rs");

        assert_eq!(
            set.Changes(),
            &[
                Change::Present {
                    path: "a.rs".to_owned(),
                    content: "one".to_owned(),
                },
                Change::Absent {
                    path: "b.rs".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn Test_Is_Empty_Should_Be_True_Only_Before_Anything_Is_Added()
    {
        let empty = WorkspaceChangeSet::From(ChangeSource::IdeEdit);
        let non_empty = empty.clone().Present("a.rs", "x");

        assert!(empty.Is_Empty());
        assert!(!non_empty.Is_Empty());
    }
}
