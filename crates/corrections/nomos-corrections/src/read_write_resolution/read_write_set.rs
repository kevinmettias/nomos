//! `COR-EXEC-001`'s declared-or-derived read/write set for one correction operation.

use super::{DerivedProvenance, ReadWriteResolution};

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
        use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

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
