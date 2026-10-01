//! What this provider offers, and at what guarantee.

use crate::scaffolding;
use nomos_cap_goals_policy::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity};

/// This provider's own name.
///
/// Named for what it reads, the same way `nomos.repo.standards`, `nomos.repo.limits`,
/// `nomos.repo.scripting` and `nomos.repo.words` are each named for the file they read
/// rather than for a hypothetical second reader of the identical file.
pub const PROVIDER: &str = "nomos.repo.goals";

/// What this provider claims, on every axis.
///
/// [`FactVariant::Syntactic`] at the ceiling, matching [`nomos_cap_goals_policy::Ceiling`]
/// exactly: this provider reads `standards.json`'s own JSON structure with no name
/// resolution and no inference over it. There is nothing here it could resolve — which
/// subsystem serves which purpose exists nowhere but the declaration.
///
/// Both [`Assurance`] axes [`Assurance::Sound`]: a provider that read the whole of the
/// `goals`, `max_subsystems_per_goal` and `subsystems` keys has read everything there is to
/// read, and reports nothing it did not find there.
///
/// [`IncrementalGranularity::WholeWorkspace`]: one repository, one `standards.json`, one set
/// of declared purposes.
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

/// This provider's offer against [`nomos_cap_goals_policy::Capability_Contract`].
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
        use nomos_cap_goals_policy::Ceiling;

        assert!(Ceiling().Satisfies(&Declared_Guarantee()));
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        use nomos_cap_goals_policy::Capability_Contract;
        use nomos_capability::contract_testing::Declared_Registry;

        let mut registry = Declared_Registry(Capability_Contract());

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }
}
