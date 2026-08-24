//! What one correction operation reads and writes, at the strongest resolution it can be
//! stated at.

use nomos_contracts::{Guarantee, ProviderId};

/// `COR-EXEC-001`'s seven named resolution tiers, in the corpus's own order.
///
/// A read/write set is stated at exactly one of these -- the finest a caller can actually
/// justify. `Semantic` is the strongest, `Artifact` the weakest; [`crate::ChangeSet::
/// Touched`] is an `Artifact`-tier answer today (raw file paths, nothing finer), which
/// this type does not change -- it only gives a caller with something stronger a place to
/// say so.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReadWriteResolution
{
    /// A whole file or generated unit, named by path.
    Artifact,
    /// One named symbol inside an artifact.
    Symbol,
    /// A configuration value or key.
    Configuration,
    /// Source this operation itself generates, rather than edits directly.
    GeneratedSource,
    /// A dependency edge between artifacts, packages or crates.
    Dependency,
    /// State held by an external provider rather than by this repository.
    ProviderState,
    /// A semantic fact -- meaning resolved past syntax, such as a fact this workspace's
    /// own analysis substrate would produce.
    Semantic,
}

impl ReadWriteResolution
{
    /// The variant's stable, lowercase wire spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Artifact => "artifact",
            Self::Symbol => "symbol",
            Self::Configuration => "configuration",
            Self::GeneratedSource => "generated_source",
            Self::Dependency => "dependency",
            Self::ProviderState => "provider_state",
            Self::Semantic => "semantic",
        };
    }
}

impl core::fmt::Display for ReadWriteResolution
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

/// What backs a read/write set that was derived rather than declared directly by the
/// operation's own author.
///
/// `COR-EXEC-001`'s own four named fields. `guarantee` reuses [`nomos_contracts::
/// Guarantee`] rather than inventing a parallel type: a derived read/write set is a fact
/// a provider produced, the same shape `Guarantee`'s own doc already answers -- what a
/// provider promises about the facts it produces -- and soundness, completeness and
/// incremental granularity apply to a derived set exactly the way they apply to any other
/// derived fact in this workspace. `confidence` and `invalidation_basis` have no existing
/// type and no shape or range the corpus names, so both are plain declared strings, the
/// same placeholder pattern [`crate::CorrectionChoice`] already uses for its own
/// caller-supplied prose fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerivedProvenance
{
    provider: ProviderId,
    guarantee: Guarantee,
    confidence: String,
    invalidation_basis: String,
}

impl DerivedProvenance
{
    #[must_use]
    pub fn New(provider: ProviderId, guarantee: Guarantee, confidence: impl Into<String>, invalidation_basis: impl Into<String>) -> Self
    {
        return Self {
            provider,
            guarantee,
            confidence: confidence.into(),
            invalidation_basis: invalidation_basis.into(),
        };
    }

    #[must_use]
    pub const fn Provider(&self) -> &ProviderId
    {
        return &self.provider;
    }

    #[must_use]
    pub const fn Guarantee(&self) -> Guarantee
    {
        return self.guarantee;
    }

    #[must_use]
    pub fn Confidence(&self) -> &str
    {
        return &self.confidence;
    }

    #[must_use]
    pub fn Invalidation_Basis(&self) -> &str
    {
        return &self.invalidation_basis;
    }
}

/// `COR-EXEC-001`: "Every correction operation shall declare or derive artifact, symbol,
/// configuration, generated-source, dependency, provider-state, and semantic read/write
/// sets at the strongest available resolution. Derived sets shall identify the provider,
/// guarantee, confidence, and invalidation basis."
///
/// A caller declares this when constructing one -- the same declared-not-computed
/// boundary [`crate::CorrectionClass`] and [`crate::RollbackBoundary`] already draw. This
/// type is free-standing, referencing nothing from [`crate::CorrectionCandidate`] and not
/// threaded into its constructor, the same shape [`crate::CorrectionChoice`] and
/// [`crate::CorrectionDecision`] already took -- so no existing call site changes because
/// this exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadWriteSet
{
    resolution: ReadWriteResolution,
    entries: Vec<String>,
    derived: Option<DerivedProvenance>,
}

impl ReadWriteSet
{
    /// A set declared directly by the operation's own author, with no derivation
    /// provenance.
    #[must_use]
    pub fn Declared(resolution: ReadWriteResolution, entries: Vec<String>) -> Self
    {
        return Self {
            resolution,
            entries,
            derived: None,
        };
    }

    /// A set some other provider derived, carrying the provenance `COR-EXEC-001` requires
    /// of a derived set.
    #[must_use]
    pub fn Derived(resolution: ReadWriteResolution, entries: Vec<String>, provenance: DerivedProvenance) -> Self
    {
        return Self {
            resolution,
            entries,
            derived: Some(provenance),
        };
    }

    #[must_use]
    pub const fn Resolution(&self) -> ReadWriteResolution
    {
        return self.resolution;
    }

    #[must_use]
    pub fn Entries(&self) -> &[String]
    {
        return &self.entries;
    }

    #[must_use]
    pub fn Derived_Provenance(&self) -> Option<&DerivedProvenance>
    {
        return self.derived.as_ref();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL_RESOLUTIONS: [ReadWriteResolution; 7] = [
        ReadWriteResolution::Artifact,
        ReadWriteResolution::Symbol,
        ReadWriteResolution::Configuration,
        ReadWriteResolution::GeneratedSource,
        ReadWriteResolution::Dependency,
        ReadWriteResolution::ProviderState,
        ReadWriteResolution::Semantic,
    ];

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL_RESOLUTIONS.iter().map(|resolution| return resolution.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two resolutions share a wire spelling");
    }

    #[test]
    fn Test_A_Declared_Set_Carries_No_Provenance()
    {
        let set = ReadWriteSet::Declared(ReadWriteResolution::Symbol, vec!["nomos_corrections::Edit".to_owned()]);

        assert_eq!(set.Resolution(), ReadWriteResolution::Symbol);
        assert_eq!(set.Entries(), ["nomos_corrections::Edit"]);
        assert!(set.Derived_Provenance().is_none());
    }

    #[test]
    fn Test_A_Derived_Set_Carries_Exactly_The_Provenance_It_Was_Given()
    {
        use nomos_contracts::{Assurance, FactVariant, IncrementalGranularity};

        let guarantee = Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
        let provenance = DerivedProvenance::New(ProviderId::New("nomos-lang-rust"), guarantee, "high", "invalidated when the crate's own dependency graph changes");

        let set = ReadWriteSet::Derived(ReadWriteResolution::Dependency, vec!["nomos-corrections -> nomos-workspace".to_owned()], provenance);

        let derived = set.Derived_Provenance().expect("a derived set carries its provenance");
        assert_eq!(derived.Provider(), &ProviderId::New("nomos-lang-rust"));
        assert_eq!(derived.Guarantee(), guarantee);
        assert_eq!(derived.Confidence(), "high");
        assert_eq!(derived.Invalidation_Basis(), "invalidated when the crate's own dependency graph changes");
    }
}
