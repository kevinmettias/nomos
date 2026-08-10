//! One consequence of applying a change.

/// What one change actually did.
///
/// Reported rather than assumed, because the submitter did not know. A checkout does not
/// diff before it lands and an editor's save hook does not consult the previous
/// generation, so [`Change::Present`] is a statement about the desired end state. This is
/// the answer to what it turned out to be.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Effect
{
    Added
    {
        path: String,
    },
    Modified
    {
        path: String,
    },
    Removed
    {
        path: String,
    },
    /// The change said what the workspace already said.
    ///
    /// Not an error and not silence. An editor saving an unmodified file and a checkout
    /// landing where you already were both arrive here, and a workspace that treated them
    /// as changes would advance a generation and invalidate every fact in the store to
    /// reach the answer it already had.
    Redundant
    {
        path: String,
    },
    /// A removal of something that was not there.
    ///
    /// Distinct from `Redundant` because it is worth seeing: a submitter deleting files
    /// the workspace never had is usually a submitter working from a different idea of
    /// what the workspace contains.
    AlreadyAbsent
    {
        path: String,
    },
}

impl Effect
{
    #[must_use]
    pub fn Path(&self) -> &str
    {
        return match self
        {
            Self::Added { path }
            | Self::Modified { path }
            | Self::Removed { path }
            | Self::Redundant { path }
            | Self::AlreadyAbsent { path } => path,
        };
    }

    /// Whether this effect changed what the workspace is.
    #[must_use]
    pub const fn Altered(&self) -> bool
    {
        return matches!(self, Self::Added { .. } | Self::Modified { .. } | Self::Removed { .. });
    }
}
