//! What a provider promises about the facts it produces.

use serde::{Deserialize, Serialize};

const SYNTACTIC_LABEL: &str = "Syntactic";
const SEMANTICALLY_RESOLVED_LABEL: &str = "SemanticallyResolved";
const RUNTIME_OBSERVED_LABEL: &str = "RuntimeObserved";
const APPROXIMATE_LABEL: &str = "Approximate";
const PREDICTED_LABEL: &str = "Predicted";

/// The resolution level at which a fact was established.
///
/// Ordered weakest first. A rule that needs name resolution cannot accept a syntactic
/// answer, and the comparison that decides so is this ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FactVariant
{
    /// Modelled rather than measured.
    Predicted,
    /// Established by a method that trades accuracy for cost.
    Approximate,
    /// Read from the text or its parse tree, with no name resolution.
    Syntactic,
    /// Established with resolved names, types and references.
    SemanticallyResolved,
    /// Established by observing execution.
    RuntimeObserved,
}

impl FactVariant
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Syntactic => SYNTACTIC_LABEL,
            Self::SemanticallyResolved => SEMANTICALLY_RESOLVED_LABEL,
            Self::RuntimeObserved => RUNTIME_OBSERVED_LABEL,
            Self::Approximate => APPROXIMATE_LABEL,
            Self::Predicted => PREDICTED_LABEL,
        };
    }
}

impl core::fmt::Display for FactVariant
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

const SOUND_LABEL: &str = "Sound";
const UNSOUND_LABEL: &str = "Unsound";
const UNKNOWN_LABEL: &str = "Unknown";

/// Whether a provider claims a formal property of its output.
///
/// Three states, not two. [`Assurance::Unknown`] is the honest answer for most real
/// analyzers and it must not be spelled the same as [`Assurance::Unsound`] — one says
/// "this reports things that are not there", the other says "nobody has established
/// either way", and a consumer's response to them differs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Assurance
{
    /// The property is claimed and the claim is backed.
    Sound,
    /// The property is known not to hold.
    Unsound,
    /// Nobody has established this either way.
    Unknown,
}

impl Assurance
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Sound => SOUND_LABEL,
            Self::Unsound => UNSOUND_LABEL,
            Self::Unknown => UNKNOWN_LABEL,
        };
    }

    /// Whether this assurance satisfies a requirement for the property.
    ///
    /// Only [`Assurance::Sound`] does. `Unknown` deliberately does not, because a
    /// requirement met by an absence of information is not a requirement.
    #[must_use]
    pub const fn Satisfies_Requirement(self) -> bool
    {
        return matches!(self, Self::Sound);
    }
}

impl core::fmt::Display for Assurance
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

const NONE_GRANULARITY_LABEL: &str = "None";
const WHOLE_WORKSPACE_LABEL: &str = "WholeWorkspace";
const PROJECT_LABEL: &str = "Project";
const FILE_LABEL: &str = "File";
const SYMBOL_LABEL: &str = "Symbol";
const REGION_LABEL: &str = "Region";

/// The finest granularity at which a provider can update its output incrementally.
///
/// The invalidation engine computes the logically minimal affected region and then
/// **broadens** it to whatever the selected provider can actually deliver. Recording
/// this per provider is what stops the engine from assuming a precision no tool has —
/// a compiler-backed provider that can only refresh a whole project is common, and
/// asking it for a symbol-level update silently gets you a stale answer.
///
/// Ordered coarsest first, so broadening is a `max`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IncrementalGranularity
{
    /// No incremental capability. Any change means a full recomputation.
    None,
    /// The whole workspace refreshes together.
    WholeWorkspace,
    /// One project or compilation unit refreshes together.
    Project,
    /// One file refreshes independently.
    File,
    /// One symbol refreshes independently.
    Symbol,
    /// A sub-symbol region refreshes independently.
    Region,
}

impl IncrementalGranularity
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::None => NONE_GRANULARITY_LABEL,
            Self::WholeWorkspace => WHOLE_WORKSPACE_LABEL,
            Self::Project => PROJECT_LABEL,
            Self::File => FILE_LABEL,
            Self::Symbol => SYMBOL_LABEL,
            Self::Region => REGION_LABEL,
        };
    }

    /// The granularity actually achievable when a region computed at `self` must be
    /// refreshed by a provider capable of `provider`.
    ///
    /// Always the coarser of the two. Asking for finer than a provider offers does not
    /// get you finer; it gets you wrong.
    #[must_use]
    pub fn Broadened_To(self, provider: Self) -> Self
    {
        return core::cmp::min(self, provider);
    }
}

impl core::fmt::Display for IncrementalGranularity
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

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

    #[test]
    fn Test_Broadening_Should_Take_The_Coarser_Granularity()
    {
        assert_eq!(
            IncrementalGranularity::Region.Broadened_To(IncrementalGranularity::Project),
            IncrementalGranularity::Project
        );
        assert_eq!(
            IncrementalGranularity::Project.Broadened_To(IncrementalGranularity::Region),
            IncrementalGranularity::Project
        );
        assert_eq!(
            IncrementalGranularity::Symbol.Broadened_To(IncrementalGranularity::None),
            IncrementalGranularity::None
        );
    }
}
