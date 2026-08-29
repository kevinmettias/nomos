//! Which of the five things a change turned out to be.

/// Which of the five things a change turned out to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectKind
{
    Added,
    Modified,
    Removed,
    /// The change said what the workspace already said.
    ///
    /// Not an error and not silence. An editor saving an unmodified file and a checkout
    /// landing where you already were both arrive here, and a workspace that treated them
    /// as changes would advance a generation and invalidate every fact in the store to
    /// reach the answer it already had.
    Redundant,
    /// A removal of something that was not there.
    ///
    /// Distinct from `Redundant` because it is worth seeing: a submitter deleting files
    /// the workspace never had is usually a submitter working from a different idea of
    /// what the workspace contains.
    AlreadyAbsent,
}

impl EffectKind
{
    /// Whether an outcome of this kind changed what the workspace is.
    #[must_use]
    pub const fn Is_Altered(self) -> bool
    {
        return matches!(self, Self::Added | Self::Modified | Self::Removed);
    }
}
