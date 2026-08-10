//! What a provider promises about the facts it produces for one capability.

use serde::{Deserialize, Serialize};

use crate::{Assurance, FactVariant, IncrementalGranularity};

/// What a provider promises about the facts it produces for one capability.
///
/// Note what this type is *not*: it is not a reproducibility claim. That is the
/// [`super::Strategy`] triple, and the two are orthogonal — a syntactic provider can be
/// bit-reproducible across platforms while a semantically-resolved one is only
/// reproducible within a single run. Merging them would force one to be reported
/// wrongly whenever they disagree, which is most of the time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Guarantee
{
    /// The resolution level at which facts are established.
    pub variant: FactVariant,
    /// Whether the provider claims to report nothing that is not there.
    pub soundness: Assurance,
    /// Whether the provider claims to report everything that is there.
    pub completeness: Assurance,
    /// The finest granularity at which the provider can refresh incrementally.
    pub incremental: IncrementalGranularity,
}

impl Guarantee
{
    /// Constructs a guarantee.
    #[must_use]
    pub const fn New(
        variant: FactVariant,
        soundness: Assurance,
        completeness: Assurance,
        incremental: IncrementalGranularity,
    ) -> Self
    {
        return Self {
            variant,
            soundness,
            completeness,
            incremental,
        };
    }

    /// Whether this guarantee is at least as strong as `required` on every axis.
    ///
    /// Every axis, not a score. A provider that is sound but syntactic does not satisfy
    /// a requirement for semantic resolution however sound it is, and a weighted
    /// average would let it.
    #[must_use]
    pub fn Satisfies(&self, required: &Self) -> bool
    {
        if self.variant < required.variant
        {
            return false;
        }
        if required.soundness.Satisfies_Requirement() && !self.soundness.Satisfies_Requirement()
        {
            return false;
        }
        if required.completeness.Satisfies_Requirement()
            && !self.completeness.Satisfies_Requirement()
        {
            return false;
        }
        return self.incremental >= required.incremental;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Semantic_And_Sound() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    #[test]
    fn Test_Guarantee_Should_Satisfy_An_Equal_Requirement()
    {
        assert!(Semantic_And_Sound().Satisfies(&Semantic_And_Sound()));
    }

    /// The case the whole type exists for: a syntactic provider must not be allowed to
    /// answer a question that needs resolved names, no matter how sound it is.
    #[test]
    fn Test_Syntactic_Should_Not_Satisfy_A_Semantic_Requirement()
    {
        let syntactic = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::Region,
        );

        assert!(!syntactic.Satisfies(&Semantic_And_Sound()));
    }

    /// An unestablished property must not satisfy a requirement for that property.
    /// This is the same rule as "unknown is not pass", one level down.
    #[test]
    fn Test_Unknown_Soundness_Should_Not_Satisfy_A_Soundness_Requirement()
    {
        let unknown_soundness = Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Unknown,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );

        assert!(!unknown_soundness.Satisfies(&Semantic_And_Sound()));
    }

    /// A requirement that does not ask for soundness must not be failed by a provider
    /// that happens not to claim it — otherwise every requirement implicitly demands
    /// every property.
    #[test]
    fn Test_Unrequired_Properties_Should_Not_Be_Demanded()
    {
        let required = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Unknown,
            Assurance::Unknown,
            IncrementalGranularity::None,
        );
        let offered = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Unknown,
            Assurance::Unsound,
            IncrementalGranularity::None,
        );

        assert!(offered.Satisfies(&required));
    }

    #[test]
    fn Test_Coarser_Incremental_Granularity_Should_Not_Satisfy_A_Finer_Requirement()
    {
        let project_level = Guarantee::New(
            FactVariant::SemanticallyResolved,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::Project,
        );

        assert!(!project_level.Satisfies(&Semantic_And_Sound()));
    }

}
