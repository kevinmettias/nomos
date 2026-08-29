//! How much of the corpus a change touched.

/// What a census counted over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope
{
    /// The ten volumes the block manifest covers. v15.0 has no such directory at all.
    DomainVolumes,
    /// Every markdown document in the revision, wherever it lives.
    EveryMarkdown,
}

impl Scope
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::DomainVolumes => "01_authoring/domain_volumes",
            Self::EveryMarkdown => "every markdown document",
        };
    }

    /// Whether this scope reaches the document at `path`.
    ///
    /// Crate-visible rather than public: it is how the census walk decides what to count,
    /// and a caller outside that walk asking it would be deciding for itself what a scope
    /// means.
    pub(crate) fn Is_Covering(self, path: &str) -> bool
    {
        use crate::DOMAIN_VOLUMES;

        return match self
        {
            Self::DomainVolumes => path.contains(DOMAIN_VOLUMES),
            Self::EveryMarkdown => true,
        };
    }
}
