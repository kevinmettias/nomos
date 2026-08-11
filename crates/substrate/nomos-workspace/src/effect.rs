//! One consequence of applying a change.

/// What one change actually did, and to what.
///
/// Reported rather than assumed, because the submitter did not know. A checkout does not
/// diff before it lands and an editor's save hook does not consult the previous
/// generation, so [`Change::Present`] is a statement about the desired end state. This is
/// the answer to what it turned out to be.
///
/// The path is the type's and not the kind's. Every consequence is a consequence *to a
/// member*, so a caller reads `path` without matching on an outcome it does not otherwise
/// care about, and a new outcome cannot forget to say which member it happened to.
///
/// `path` is declared first so that ordering a set of effects orders it by member. The
/// order effects were applied in is not a fact about the workspace — it is a fact about
/// how the change set was iterated — and sorting by outcome would group by exactly the
/// thing the caller is usually reading past.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Effect
{
    pub path: String,
    pub kind: EffectKind,
}

impl Effect
{
    #[must_use]
    pub fn Path(&self) -> &str
    {
        return &self.path;
    }

    /// Whether this effect changed what the workspace is.
    #[must_use]
    pub const fn Altered(&self) -> bool
    {
        return self.kind.Altered();
    }
}

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
    pub const fn Altered(self) -> bool
    {
        return matches!(self, Self::Added | Self::Modified | Self::Removed);
    }
}
