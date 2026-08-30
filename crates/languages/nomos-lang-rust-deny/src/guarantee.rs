//! What this provider offers, and at what guarantee.

use nomos_cap_dependency_policy::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This provider's own name.
///
/// Named for the tool it actually runs, the same way `nomos.lang.rust.clippy` is: a
/// future provider answering the same capability from a different tool (`cargo audit`
/// wrapped for the same purpose, a second-language policy tool) would need to disagree
/// with this name honestly.
pub const PROVIDER: &str = "nomos.lang.rust.deny";

/// What this provider claims, on every axis.
///
/// [`FactVariant::SemanticallyResolved`] at the ceiling: `cargo deny` reasons over
/// `Cargo.lock`'s own fully resolved dependency graph, not `Cargo.toml`'s unresolved
/// ranges.
///
/// Soundness [`Assurance::Sound`]: every violation this provider reports is a violation
/// `cargo deny` itself emitted; nothing here infers or synthesizes one.
///
/// Completeness [`Assurance::Unknown`], deliberately not `Sound` — this workspace's own
/// `deny.toml` is what `cargo deny` actually checks against, and a category that file does
/// not restrict (an unbanned crate, a license this repository simply has no dependency
/// carrying yet) is not something this provider can honestly claim it would have caught.
/// The same "no floor no provider can honestly meet" reasoning `OD-RULES-010`'s own text
/// already gives for `cargo clippy` on this identical axis, for a different underlying
/// reason: there the tool's own lints are incomplete by design, here the tool is only as
/// complete as a policy file it does not author.
///
/// [`IncrementalGranularity::WholeWorkspace`]: this provider materializes exactly one fact
/// for the resolved dependency graph as a whole — `nomos-cap-dependency-policy`'s own
/// ceiling states why no per-member split is honestly available.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::WholeWorkspace,
    );
}

/// This provider's offer against [`nomos_cap_dependency_policy::Capability_Contract`].
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
    /// offer.guarantee)` is the exact check `Registry::Offer` runs at composition time,
    /// asserted here in the same direction so a future weakening of either constant is
    /// caught beside the constants rather than only at whatever composition root happens
    /// to run first.
    #[test]
    fn Test_The_Ceiling_Should_Satisfy_The_Declared_Guarantee()
    {
        use nomos_cap_dependency_policy::Ceiling;

        assert!(Ceiling().Satisfies(&Declared_Guarantee()));
    }

    /// `Provider_Offer` itself, addressed by name rather than only through the ceiling
    /// property above -- the same composition-time check
    /// `tests/integration_seams.rs::Test_Provider_Offer_Should_Be_Accepted_By_Nomos_
    /// Capabilitys_Own_Registry` repeats through this crate's public API, kept here as well
    /// because a Rust test outside this file cannot address a function declared in it.
    #[test]
    fn Test_Provider_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        use nomos_cap_dependency_policy::Capability_Contract;
        use nomos_capability::Registry;

        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }
}
