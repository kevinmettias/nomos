//! What this provider offers, and at what guarantee.

use nomos_cap_limits_policy::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This provider's own name.
///
/// Named for what it reads, the same way `nomos.repo.standards` is named for
/// `standards.json` rather than for a hypothetical second reader of the identical file: a
/// future provider reading a different configuration surface for the same capability
/// would need to disagree with this name honestly.
pub const PROVIDER: &str = "nomos.repo.limits";

/// What this provider claims, on every axis.
///
/// [`FactVariant::Syntactic`] at the ceiling, matching [`nomos_cap_limits_policy::Ceiling`]
/// exactly: this provider reads `standards.json`'s own JSON structure with no name
/// resolution and no inference over it.
///
/// Both [`Assurance`] axes [`Assurance::Sound`]: every row this provider reports is a
/// threshold `standards.json` actually declares, and a provider that read the whole of the
/// `limits` and `languages.*.limits` blocks has read everything there is to read.
///
/// [`IncrementalGranularity::WholeWorkspace`]: one repository, one `standards.json`, one
/// declared set of thresholds.
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

/// This provider's offer against [`nomos_cap_limits_policy::Capability_Contract`].
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

    /// The offer must satisfy the contract's own ceiling — `contract.ceiling.Satisfies(&
    /// offer.guarantee)` is the exact check `Registry::Offer` runs at composition time.
    #[test]
    fn Test_The_Ceiling_Should_Satisfy_The_Declared_Guarantee()
    {
        use nomos_cap_limits_policy::Ceiling;

        assert!(Ceiling().Satisfies(&Declared_Guarantee()));
    }

    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        use nomos_cap_limits_policy::Capability_Contract;
        use nomos_capability::Registry;

        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }
}
