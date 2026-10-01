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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Label_Should_Describe_Each_Scope()
    {
        assert_eq!(Scope::DomainVolumes.Label(), "01_authoring/domain_volumes");
        assert_eq!(Scope::EveryMarkdown.Label(), "every markdown document");
    }

    #[test]
    fn Test_Is_Covering_Should_Restrict_Domain_Volumes_But_Not_Every_Markdown()
    {
        assert!(Scope::DomainVolumes.Is_Covering("01_authoring/domain_volumes/02-core/a.md"));
        assert!(!Scope::DomainVolumes.Is_Covering("09-reference/glossary.md"));
        assert!(Scope::EveryMarkdown.Is_Covering("09-reference/glossary.md"));
    }
}
