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
