//! What this provider offers, and at what guarantee.

use crate::scaffolding;
use nomos_cap_architecture::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

/// This provider's own name.
///
/// Named for what it reads, the same way `nomos.repo.standards`, `nomos.repo.limits`,
/// `nomos.repo.scripting`, `nomos.repo.words` and `nomos.repo.goals` each are. A second
/// provider reading a different declaration surface -- a manifest, a dedicated file -- would
/// need to disagree with this name honestly.
pub const PROVIDER: &str = "nomos.repo.architecture";

/// What this provider claims, on every axis.
///
/// [`FactVariant::Syntactic`] at the ceiling, matching [`nomos_cap_architecture::Ceiling`]
/// exactly: this provider reads `standards.json`'s own JSON structure with no name resolution
/// and no inference over it. There is nothing here it could resolve -- which component a
/// package belongs to exists nowhere but the declaration, which is the whole point of
/// `OD-RULES-024`'s "the missing fact is a declared architecture, not an inference engine".
///
/// Both [`Assurance`] axes [`Assurance::Sound`]: a provider that read the whole of the
/// `architecture` block has read everything there is to read, and reports nothing it did not
/// find there.
///
/// [`IncrementalGranularity::WholeWorkspace`]: one repository, one `standards.json`, one
/// architecture. The order over components is a property of no single member.
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

/// This provider's offer against [`nomos_cap_architecture::Capability_Contract`].
#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return scaffolding::Offer(PROVIDER, Capability(), CONTRACT_VERSION, Declared_Guarantee());
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The offer must satisfy the contract's own ceiling -- `contract.ceiling.Satisfies(&
    /// offer.guarantee)` is the exact check `Registry::Offer` runs at composition time.
    #[test]
    fn Test_The_Ceiling_Should_Satisfy_The_Declared_Guarantee()
    {
        use nomos_cap_architecture::Ceiling;

        assert!(Ceiling().Satisfies(&Declared_Guarantee()));
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        use nomos_cap_architecture::Capability_Contract;
        use nomos_capability::contract_testing::Declared_Registry;

        let mut registry = Declared_Registry(Capability_Contract());

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }
}
