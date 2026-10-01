//! What backs a read/write set that was derived rather than declared directly by the
//! operation's own author.

use nomos_contracts::{Guarantee, ProviderId};

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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Assurance, FactVariant, IncrementalGranularity};

    #[test]
    fn Test_New_Should_Construct_A_Value_From_Its_Given_Arguments()
    {
        let provenance = DerivedProvenance::New(ProviderId::New("nomos-lang-rust"), Sample_Guarantee(), "high", "reason");

        assert_eq!(provenance.Provider(), &ProviderId::New("nomos-lang-rust"));
        assert_eq!(provenance.Invalidation_Basis(), "reason");
    }

    #[test]
    fn Test_Provider_Should_Report_Who_Produced_The_Fact()
    {
        let provenance = DerivedProvenance::New(ProviderId::New("nomos-lang-rust"), Sample_Guarantee(), "high", "reason");

        assert_eq!(provenance.Provider(), &ProviderId::New("nomos-lang-rust"));
    }

    #[test]
    fn Test_Guarantee_Should_Report_What_The_Provider_Promised()
    {
        let guarantee = Sample_Guarantee();
        let provenance = DerivedProvenance::New(ProviderId::New("nomos-lang-rust"), guarantee, "high", "reason");

        assert_eq!(provenance.Guarantee(), guarantee);
    }

    #[test]
    fn Test_Confidence_Should_Report_The_Callers_Own_Assessment()
    {
        let provenance = DerivedProvenance::New(ProviderId::New("nomos-lang-rust"), Sample_Guarantee(), "high", "reason");

        assert_eq!(provenance.Confidence(), "high");
    }

    #[test]
    fn Test_Invalidation_Basis_Should_Report_When_The_Fact_Goes_Stale()
    {
        let provenance = DerivedProvenance::New(ProviderId::New("nomos-lang-rust"), Sample_Guarantee(), "high", "the dependency graph changed");

        assert_eq!(provenance.Invalidation_Basis(), "the dependency graph changed");
    }

    fn Sample_Guarantee() -> Guarantee
    {
        return Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    }
}
