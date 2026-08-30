//! What this provider promises, stated as a value and checked in two directions.
//!
//! Downward, [`nomos_capability::Registry`] refuses [`Provider_Offer`] if it claims more
//! than [`Capability_Contract`]'s ceiling permits — a provider does not get to grade its
//! own work. Upward, `tests/guarantee.rs` asserts one property per axis against what the
//! provider actually emits, so the declaration is not merely permitted but true.

use nomos_cap_syntax::{Capability, CONTRACT_VERSION};
use nomos_capability::ProviderOffer;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};

/// This implementation.
///
/// The tool is in the name because the tool is a fact about the answer. When a second
/// Rust provider exists — one backed by a compiler, offering
/// [`FactVariant::SemanticallyResolved`] — a caller comparing two results needs to see
/// which one it is looking at without consulting a table.
pub const PROVIDER: &str = "nomos.lang.rust.syn";

/// What this provider claims, on every axis.
///
/// # Syntactic, and not by modesty
///
/// Every answer here is a function of one file's bytes. Nothing resolves a name, follows
/// a `use`, or knows whether the `Display` in `impl Display for Foo` is the one in
/// `std::fmt`. A rule that needs to know cannot be satisfied by this provider, and
/// [`Guarantee::Satisfies`] is what stops it from being.
///
/// # Sound
///
/// Every item reported was in the token stream. `syn` does not infer items, so the
/// provider has no path by which to report one the file does not contain — which is what
/// makes this claimable rather than hoped for.
///
/// # Completeness is Unknown, and this is the honest answer
///
/// Not [`Assurance::Unsound`], which would say the output is known to omit things in a
/// characterized way, and emphatically not [`Assurance::Sound`]. A macro invocation is
/// where the parse tree ends and an unexpanded token stream begins, and items generated
/// there are invisible here. Worse, the count of such places is itself a lower bound:
/// `#[tokio::main]` and `#[allow(dead_code)]` are syntactically indistinguishable, and
/// telling them apart is name resolution — the thing this provider does not do.
///
/// So the size of the gap is not known, and a provider that cannot bound its own
/// omissions may not claim completeness. [`Facts::unexpanded`] reports the lower
/// bound so a caller can see how much of a file was beyond reach rather than inferring
/// silence from a small number.
///
/// [`Facts::unexpanded`]: crate::Facts::unexpanded
///
/// # File
///
/// A Rust file parses without reference to any other file, so a change to one file
/// requires reparsing exactly that file. Not `Symbol`: `syn` parses a whole file or
/// fails, and there is no partial reparse to offer. Claiming `Symbol` would let the
/// invalidation engine refresh one function and believe the rest of the file current.
#[must_use]
pub const fn Declared_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

/// This provider's offer against [`nomos_cap_syntax::Capability_Contract`].
///
/// The capability, its version and the ceiling this is checked against are all
/// `nomos-cap-syntax`'s. What is decided here is only what this implementation promises,
/// which is the one part of the arrangement a provider is entitled to state about itself.
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
    use nomos_cap_syntax::Capability_Contract;
    use nomos_capability::{OfferRefusal, Registry, RegistryError, RegistryErrorKind, Requirement};

    /// Not "its own contract". This provider does not author the terms it offers under,
    /// and every test below declares them from `nomos-cap-syntax` for that reason.
    #[test]
    fn Test_The_Offer_Should_Be_Accepted_Under_The_Capabilitys_Contract()
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
    }

    /// The reason the ceiling exists. A syntactic provider that could claim resolution
    /// would satisfy every rule that requires it, and each of those rules would then be
    /// accepting an answer that cannot support it.
    #[test]
    fn Test_Claiming_Resolution_Should_Be_Refused()
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");

        let refused = registry.Offer(Claiming_Resolution());

        assert_eq!(
            refused,
            Err(RegistryError {
                capability: Capability(),
                kind: RegistryErrorKind::Offer {
                    provider: ProviderId::New(PROVIDER),
                    refusal: OfferRefusal::ExceedsCeiling,
                },
            })
        );
    }

    /// This provider's offer, with the variant raised past what its parse can support.
    fn Claiming_Resolution() -> ProviderOffer
    {
        return ProviderOffer {
            guarantee: Guarantee::New(
                FactVariant::SemanticallyResolved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::File,
            ),
            ..Provider_Offer()
        };
    }

    /// The consequence of `Unknown` completeness, made visible at the resolution site
    /// rather than left as documentation. A caller that needs to know it has seen every
    /// item does not get this provider, and does not get a weaker answer silently.
    /// A registry holding this provider's contract and its offer, and nothing else.
    ///
    /// The composition every test below asks a question of. Built once, because a test that
    /// registered a different composition from its neighbour would be answering about a
    /// registry nobody ships.
    fn Serving() -> Registry
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");
        registry
            .Offer(Provider_Offer())
            .expect("the offer is within the ceiling");

        return registry;
    }

    #[test]
    fn Test_A_Caller_Needing_Completeness_Should_Not_Resolve_To_This_Provider()
    {
        let registry = Serving();

        let every_item = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Unknown,
            Assurance::Sound,
            IncrementalGranularity::None,
        );
        let needs_every_item = Requirement::New(Capability(), CONTRACT_VERSION, every_item);

        assert!(
            registry.Resolve(&needs_every_item).Offer().is_none(),
            "a rule that must see every item cannot be served by a provider that cannot \
             bound what macros hid from it"
        );
    }

    /// The positive control for the test above. If resolution refused everything, that
    /// test would pass over a registry that serves nobody.
    #[test]
    fn Test_A_Caller_Needing_Only_Soundness_Should_Resolve_To_This_Provider()
    {
        let registry = Serving();

        let what_is_there = Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
        let needs_what_is_there = Requirement::New(Capability(), CONTRACT_VERSION, what_is_there);

        let resolved = registry.Resolve(&needs_what_is_there);

        assert_eq!(
            resolved.Offer().map(|offer| return offer.provider.clone()),
            Some(ProviderId::New(PROVIDER))
        );
    }
}
