//! One path's before and after, which is what makes a change reversible.

use nomos_workspace::Change;

/// One path a correction touches, carrying both directions.
///
/// `nomos_workspace::Change` carries only what a path becomes, because the workspace does
/// not retain content for a caller to reverse against — only digests. A correction has to
/// be undoable, so an `Edit` carries what a path held before as well as what it should hold
/// after, and [`Edit::Reverse`] is then a plain swap rather than a lookup that might fail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit
{
    path: String,
    before: Option<String>,
    after: Option<String>,
}

impl Edit
{
    /// A path moving from `before` to `after`. `None` means the path does not exist in
    /// that direction.
    #[must_use]
    pub fn New(path: impl Into<String>, before: Option<String>, after: Option<String>) -> Self
    {
        return Self {
            path: path.into(),
            before,
            after,
        };
    }

    #[must_use]
    pub fn Path(&self) -> &str
    {
        return &self.path;
    }

    #[must_use]
    pub fn Before(&self) -> Option<&str>
    {
        return self.before.as_deref();
    }

    #[must_use]
    pub fn After(&self) -> Option<&str>
    {
        return self.after.as_deref();
    }

    /// What this edit does, as a change the workspace door accepts.
    #[must_use]
    pub fn Forward(&self) -> Change
    {
        return Change_Toward(&self.path, self.after.as_deref());
    }

    /// What undoes this edit, as a change the workspace door accepts.
    #[must_use]
    pub fn Reverse(&self) -> Change
    {
        return Change_Toward(&self.path, self.before.as_deref());
    }
}

fn Change_Toward(path: &str, content: Option<&str>) -> Change
{
    return match content
    {
        Some(content) => Change::Present {
            path: path.to_owned(),
            content: content.to_owned(),
        },
        None => Change::Absent {
            path: path.to_owned(),
        },
    };
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
