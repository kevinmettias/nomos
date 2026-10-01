//! What this provider offers, and at what guarantee.

use crate::scaffolding;
use nomos_cap_standards_corpus::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

/// This provider's own name.
///
/// Named for the capability rather than for the file, the same way `nomos.repo.test-material`
/// is named for what it reads: one provider answers one capability, and this capability's name
/// is the thing a second reader of the identical file would have to displace.
pub const PROVIDER: &str = "nomos.repo.standards-corpus";

/// What this provider claims, on every axis.
///
/// [`FactVariant::Syntactic`] at the ceiling, matching
/// [`nomos_cap_standards_corpus::Ceiling`] exactly: this provider reads each document's own
/// delimiter-fenced front matter as text. Nothing here resolves a name, and nothing here reads
/// the prose below the fence — what a rule's body *means* is not a question this fact answers,
/// which is why a reader claiming [`FactVariant::SemanticallyResolved`] would satisfy rules
/// this one cannot.
///
/// Both [`Assurance`] axes [`Assurance::Sound`]: a provider that walked every declared root to
/// exhaustion has read every document there is to read, and reports nothing it did not find
/// there. A root it could not enumerate is an issue in the payload rather than a silently
/// smaller population.
///
/// [`IncrementalGranularity::WholeWorkspace`]: one repository, one declaration, one population.
/// A second corpus a repository points at is a second root in the same declaration rather than
/// a second fact.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::WholeWorkspace,
    );
}

/// This provider's offer against [`nomos_cap_standards_corpus::Capability_Contract`].
#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return scaffolding::Provider_Offer(PROVIDER, Capability(), CONTRACT_VERSION, Declared_Guarantee());
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The offer must satisfy the contract's own ceiling — `contract.ceiling.Satisfies(&
    /// offer.guarantee)` is the exact check `Registry::Offer` runs at composition time.
    #[test]
    fn Test_The_Ceiling_Should_Satisfy_The_Declared_Guarantee()
    {
        use nomos_cap_standards_corpus::Ceiling;

        assert!(Ceiling().Satisfies(&Declared_Guarantee()));
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        use nomos_cap_standards_corpus::Capability_Contract;
        use nomos_capability::contract_testing::Declared_Registry;

        let mut registry = Declared_Registry(Capability_Contract());

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }
}
