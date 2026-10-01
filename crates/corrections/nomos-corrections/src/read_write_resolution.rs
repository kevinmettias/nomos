//! `COR-EXEC-001`'s seven named resolution tiers a read/write set can be stated at.

mod derived_provenance;
mod read_write_set;

pub use derived_provenance::DerivedProvenance;
pub use read_write_set::ReadWriteSet;

/// `COR-EXEC-001`'s seven named resolution tiers, in the corpus's own order.
///
/// A read/write set is stated at exactly one of these -- the finest a caller can actually
/// justify. `Semantic` is the strongest, `Artifact` the weakest; [`crate::ChangeSet::
/// Touched`] is an `Artifact`-tier answer today (raw file paths, nothing finer), which
/// this type does not change -- it only gives a caller with something stronger a place to
/// say so.
///
/// Ordered as declared, weakest first, so that a subject keyed by its tier sorts by tier
/// before spelling -- the order [`crate::Compatibility`] reports overlaps in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    fn Test_Label_Should_Be_Distinct_Per_Variant()
    {
        let mut labels: Vec<&str> = ALL_RESOLUTIONS.iter().map(|resolution| return resolution.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two resolutions share a wire spelling");
    }
}
