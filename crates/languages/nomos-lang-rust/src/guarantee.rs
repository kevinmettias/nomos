//! What this provider promises, stated as a value and checked in two directions.
//!
//! Downward, [`nomos_capability::Registry`] refuses [`Provider_Offer`] if it claims more
//! than [`Capability_Contract`]'s ceiling permits — a provider does not get to grade its
//! own work. Upward, `tests/guarantee.rs` asserts one property per axis against what the
//! provider actually emits, so the declaration is not merely permitted but true.

use nomos_capability::{CapabilityContract, ProviderOffer};
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    ProviderId, SchemaId,
};

/// The capability this provider offers.
///
/// Named for what a caller gets — the items a file declares — rather than for how it is
/// obtained. `nomos.cap.syn.parse` would make the contract a description of this
/// implementation, and the second provider of the same capability could not honestly
/// offer it.
pub const CAPABILITY: &str = "nomos.cap.syntax.items";

/// This implementation.
///
/// The tool is in the name because the tool is a fact about the answer. When a second
/// Rust provider exists — one backed by a compiler, offering
/// [`FactVariant::SemanticallyResolved`] — a caller comparing two results needs to see
/// which one it is looking at without consulting a table.
pub const PROVIDER: &str = "nomos.lang.rust.syn";

/// The payload schema. Versioned separately from the contract because the shape of the
/// bytes and the meaning of the capability change for different reasons.
pub const SCHEMA: &str = "nomos.syntax.items.v1";

/// The contract version. Not the crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

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
/// omissions may not claim completeness. [`SyntaxFacts::unexpanded`] reports the lower
/// bound so a caller can see how much of a file was beyond reach rather than inferring
/// silence from a small number.
///
/// [`SyntaxFacts::unexpanded`]: crate::SyntaxFacts::unexpanded
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

/// The capability's ceiling — the strongest anything may claim for it.
///
/// [`FactVariant::Syntactic`] is the ceiling because the capability is about what a file
/// says on its face. A compiler-backed provider that resolves names is answering a
/// different question and belongs behind a different contract; letting it offer this one
/// at [`FactVariant::SemanticallyResolved`] would mean two providers of one capability
/// disagreeing about what the capability means.
///
/// Completeness and granularity are *not* pinned to what this provider achieves.
/// [`Assurance::Sound`] and [`IncrementalGranularity::Region`] leave room for a provider
/// that expands macros or reparses incrementally. A ceiling set to today's best
/// implementation is a ceiling that has to be raised every time somebody improves
/// something, and a ceiling that moves is not a ceiling.
#[must_use]
pub fn Capability_Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: CapabilityId::New(CAPABILITY),
        version: CONTRACT_VERSION,
        summary: "The items a source file declares, as written, with the visibility each \
                  one declares and a count of the places the parse tree ends in \
                  unexpanded tokens."
            .to_owned(),
        ceiling: Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::Region,
        ),
    };
}

/// This provider's offer against [`Capability_Contract`].
#[must_use]
pub fn Provider_Offer() -> ProviderOffer
{
    return ProviderOffer {
        provider: ProviderId::New(PROVIDER),
        capability: CapabilityId::New(CAPABILITY),
        version: CONTRACT_VERSION,
        guarantee: Declared_Guarantee(),
    };
}

/// The schema every payload this provider writes is stamped with.
#[must_use]
pub fn Payload_Schema() -> SchemaId
{
    return SchemaId::New(SCHEMA);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_capability::{Registry, RegistryError, Requirement};

    #[test]
    fn Test_The_Offer_Should_Be_Accepted_Under_Its_Own_Contract()
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

        let overreaching = ProviderOffer {
            guarantee: Guarantee::New(
                FactVariant::SemanticallyResolved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::File,
            ),
            ..Provider_Offer()
        };

        assert_eq!(
            registry.Offer(overreaching),
            Err(RegistryError::ExceedsCeiling {
                capability: CapabilityId::New(CAPABILITY),
                provider: ProviderId::New(PROVIDER),
            })
        );
    }

    /// The consequence of `Unknown` completeness, made visible at the resolution site
    /// rather than left as documentation. A caller that needs to know it has seen every
    /// item does not get this provider, and does not get a weaker answer silently.
    #[test]
    fn Test_A_Caller_Needing_Completeness_Should_Not_Resolve_To_This_Provider()
    {
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");
        registry
            .Offer(Provider_Offer())
            .expect("the offer is within the ceiling");

        let needs_every_item = Requirement::New(
            CapabilityId::New(CAPABILITY),
            CONTRACT_VERSION,
            Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Unknown,
                Assurance::Sound,
                IncrementalGranularity::None,
            ),
        );

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
        let mut registry = Registry::New();
        registry
            .Declare(Capability_Contract())
            .expect("the contract is the first declaration in a fresh registry");
        registry
            .Offer(Provider_Offer())
            .expect("the offer is within the ceiling");

        let needs_what_is_there = Requirement::New(
            CapabilityId::New(CAPABILITY),
            CONTRACT_VERSION,
            Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Sound,
                Assurance::Unknown,
                IncrementalGranularity::File,
            ),
        );

        let resolved = registry.Resolve(&needs_what_is_there);

        assert_eq!(
            resolved.Offer().map(|offer| return offer.provider.clone()),
            Some(ProviderId::New(PROVIDER))
        );
    }
}
