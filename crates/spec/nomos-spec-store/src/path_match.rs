//! How a document path was matched.

/// How a document path was matched.
///
/// Reported rather than swallowed, so a caller who typed a fragment and got one document
/// can see it was a fragment that matched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathMatch
{
    /// The path was given in full.
    Exact,
    /// The last segment of the path was given.
    FileName,
    /// The text appears somewhere in the path.
    Fragment,
}

impl PathMatch
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Exact => "exact path",
            Self::FileName => "file name",
            Self::Fragment => "path fragment",
        };
    }
}
