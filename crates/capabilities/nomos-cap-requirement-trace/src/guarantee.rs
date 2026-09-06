//! What this provider offers, and at what guarantee.

use crate::contract::{Capability, Ceiling, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Guarantee, ProviderId};

/// This provider's own name.
///
/// Named for what it reads, the same `nomos.repo.*` shape `nomos-repo-policy`'s own four
/// providers use — even though this provider lives inside its own capability crate rather
/// than beside those four, it reads this repository's own committed corpus the same way
/// they read `standards.json`.
pub const PROVIDER: &str = "nomos.repo.requirement.trace";

/// What this provider claims, on every axis -- exactly [`Ceiling`], since this is the sole
/// provider and it reads every committed assessment and checks every site, gap and record it
/// names in full.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Ceiling();
}

/// This provider's offer against [`crate::contract::Capability_Contract`].
#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(PROVIDER),
        capability: Capability(),
        version: CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The offer must satisfy the contract's own ceiling — `contract.ceiling.
    /// Satisfies(&offer.guarantee)` is the exact check `Registry::Offer` runs at
    /// composition time.
    #[test]
    fn Test_The_Ceiling_Should_Satisfy_The_Declared_Guarantee()
    {
        assert!(Ceiling().Satisfies(&Declared_Guarantee()));
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        use crate::contract::Capability_Contract;
        use nomos_capability::Registry;

        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }
}
