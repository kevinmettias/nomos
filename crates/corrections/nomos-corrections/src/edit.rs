//! One path's before and after, which is what makes a change reversible.

use nomos_workspace::Change;

mod side;

use side::EditSide;

/// One path a correction touches, carrying both directions.
///
/// `nomos_workspace::Change` carries only what a path becomes, because the workspace does
/// not retain content for a caller to reverse against — only digests. A correction has to
/// be undoable, so an `Edit` carries what a path held before as well as what it should hold
/// after, and [`Edit::Reverse`] is then a plain swap rather than a lookup that might fail.
/// Each direction is an [`EditSide`], because it is the side carrying the content that
/// knows where that content goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit
{
    before: EditSide,
    after: EditSide,
}

impl Edit
{
    /// A path moving from `before` to `after`. `None` means the path does not exist in
    /// that direction.
    #[must_use]
    pub fn New(path: impl Into<String>, before: Option<String>, after: Option<String>) -> Self
    {
        let path = path.into();

        return Self {
            before: EditSide::New(path.clone(), before),
            after: EditSide::New(path, after),
        };
    }

    #[must_use]
    pub fn Path(&self) -> &str
    {
        return self.after.Path();
    }

    #[must_use]
    pub fn Before(&self) -> Option<&str>
    {
        return self.before.Content();
    }

    #[must_use]
    pub fn After(&self) -> Option<&str>
    {
        return self.after.Content();
    }

    /// What this edit does, as a change the workspace door accepts.
    #[must_use]
    pub fn Forward(&self) -> Change
    {
        return self.after.As_Change();
    }

    /// What undoes this edit, as a change the workspace door accepts.
    #[must_use]
    pub fn Reverse(&self) -> Change
    {
        return self.before.As_Change();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Reverse_Should_Undo_Forward()
    {
        let edit = Edit::New("src/a.rs", Some("old".to_owned()), Some("new".to_owned()));

        assert_eq!(
            edit.Forward(),
            Change::Present {
                path: "src/a.rs".to_owned(),
                content: "new".to_owned()
            }
        );
        assert_eq!(
            edit.Reverse(),
            Change::Present {
                path: "src/a.rs".to_owned(),
                content: "old".to_owned()
            }
        );
    }

    #[test]
    fn Test_A_Removal_Should_Reverse_To_A_Restoration()
    {
        let edit = Edit::New("src/a.rs", Some("old".to_owned()), None);

        assert_eq!(
            edit.Forward(),
            Change::Absent {
                path: "src/a.rs".to_owned()
            }
        );
        assert_eq!(
            edit.Reverse(),
            Change::Present {
                path: "src/a.rs".to_owned(),
                content: "old".to_owned()
            }
        );
    }

    #[test]
    fn Test_An_Addition_Should_Reverse_To_A_Removal()
    {
        let edit = Edit::New("src/new.rs", None, Some("content".to_owned()));

        assert_eq!(
            edit.Forward(),
            Change::Present {
                path: "src/new.rs".to_owned(),
                content: "content".to_owned()
            }
        );
        assert_eq!(
            edit.Reverse(),
            Change::Absent {
                path: "src/new.rs".to_owned()
            }
        );
    }
}
