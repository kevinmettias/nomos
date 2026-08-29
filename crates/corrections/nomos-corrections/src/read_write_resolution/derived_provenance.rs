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
