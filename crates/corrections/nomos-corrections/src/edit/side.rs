//! One direction of an edit: a path, and what it holds along that direction.

use nomos_workspace::Change;

/// A path and the content it holds in one direction — what `Edit` calls "before" when it
/// is the starting side, and "after" when it is the destination side.
///
/// Carrying `path` alongside `content`, rather than passing the path in separately at each
/// call site, is what lets [`EditSide::As_Change`] build a [`Change`] on its own: the side
/// that has the content is also the side that knows where it goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EditSide
{
    path: String,
    content: Option<String>,
}

impl EditSide
{
    #[must_use]
    pub(super) fn New(path: impl Into<String>, content: Option<String>) -> Self
    {
        return Self { path: path.into(), content };
    }

    #[must_use]
    pub(super) fn Path(&self) -> &str
    {
        return &self.path;
    }

    #[must_use]
    pub(super) fn Content(&self) -> Option<&str>
    {
        return self.content.as_deref();
    }

    /// This side, as a change the workspace door accepts.
    #[must_use]
    pub(super) fn As_Change(&self) -> Change
    {
        return match &self.content
        {
            Some(content) => Change::Present {
                path: self.path.clone(),
                content: content.clone(),
            },
            None => Change::Absent {
                path: self.path.clone(),
            },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_New_Should_Construct_A_Side_From_Its_Given_Arguments()
    {
        let side = EditSide::New("a.rs".to_owned(), Some("x".to_owned()));

        assert_eq!(side.Path(), "a.rs");
        assert_eq!(side.Content(), Some("x"));
    }

    #[test]
    fn Test_Path_Should_Report_The_Sides_Own_Location()
    {
        let side = EditSide::New("a.rs".to_owned(), None);

        assert_eq!(side.Path(), "a.rs");
    }

    #[test]
    fn Test_Content_Should_Be_None_When_This_Side_Carries_No_Value()
    {
        let side = EditSide::New("a.rs".to_owned(), None);

        assert_eq!(side.Content(), None);
    }

    #[test]
    fn Test_As_Change_Should_Produce_A_Present_Change_When_Content_Is_Set()
    {
        let side = EditSide::New("a.rs".to_owned(), Some("x".to_owned()));

        assert_eq!(
            side.As_Change(),
            Change::Present {
                path: "a.rs".to_owned(),
                content: "x".to_owned()
            }
        );
    }
}
