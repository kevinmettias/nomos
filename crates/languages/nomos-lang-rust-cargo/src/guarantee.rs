//! What this provider offers, and at what guarantee.

use nomos_cap_dependency::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{
    Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId,
};

/// This provider's own name.
///
/// Named for the tool it actually runs, the same way `nomos.lang.rust.rollup` is named
/// for its method rather than for its capability: a future provider answering the same
/// capability from a different resolution path (a build-script trace, a lockfile read
/// with no `cargo metadata` invocation) would need to disagree with this name honestly.
pub const PROVIDER: &str = "nomos.lang.rust.cargo";

/// What this provider claims, on every axis.
///
/// [`FactVariant::SemanticallyResolved`] at the ceiling: `cargo metadata` resolves each
/// declared dependency against the package it actually names — a path dependency's
/// manifest path, in particular, is Cargo's own answer to "which package is this", not
/// this provider's guess.
///
/// Soundness [`Assurance::Sound`]: every edge this provider reports is an edge Cargo's own
/// resolution reported; nothing here infers an edge from context.
///
/// Completeness [`Assurance::Sound`] as well, and this is a real claim rather than an
/// aspiration: `cargo metadata --all-features` reports every dependency a manifest
/// declares, unconditionally on which features happen to be enabled for this run, so a
/// package's edge set is not a function of how this provider was invoked. This is the one
/// axis where this provider is honestly stronger than `nomos-lang-rust`'s own syntax
/// provider, which cannot bound what a macro hid.
///
/// [`IncrementalGranularity::Project`]: a package's whole manifest is one declaration, and
/// there is no slice of "which dependency changed" cargo's own metadata exposes below
/// re-reading the manifest as a whole.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Project,
    );
}

/// This provider's offer against [`nomos_cap_dependency::Capability_Contract`].
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

// `check-test-coverage`'s Rust front end keys a function's own address-test off the
// literal file the test is textually written in -- `nomos_capability::Registry`'s own
// `registry.rs` documents the same constraint beside its `local_tests` module. A real
// crate-boundary check for `Declared_Guarantee`/`Provider_Offer` already lives in
// `tests/integration_seams.rs`, driven only through this crate's public API; this module
// is the minimal, same-file companion that check needs, not a second copy of that
// coverage.
#[cfg(test)]
mod tests
{
    use super::*;

    /// The offer must satisfy the contract's own ceiling — the property
    /// `Registry::Offer` checks at composition time, asserted here directly so a future
    /// weakening of either constant is caught beside the constants rather than only at
    /// whatever composition root happens to run first.
    #[test]
    fn Test_The_Declared_Guarantee_Should_Satisfy_The_Contract_Ceiling()
    {
        use nomos_cap_dependency::Ceiling;

        assert!(Declared_Guarantee().Satisfies(&Ceiling()));
    }

    /// `Provider_Offer`'s own fields are exactly this crate's declared identity and
    /// guarantee — the construction `tests/integration_seams.rs`'s external tests hand to
    /// `nomos_capability::Registry::Offer`, asserted here directly against the values it
    /// is built from.
    #[test]
    fn Test_Provider_Offer_Should_Carry_This_Crates_Declared_Identity_And_Guarantee()
    {
        let offer = Provider_Offer();

        assert_eq!(offer.provider, ProviderId::New(PROVIDER));
        assert_eq!(offer.capability, Capability());
        assert_eq!(offer.version, CONTRACT_VERSION);
        assert_eq!(offer.guarantee, Declared_Guarantee());
    }
}
