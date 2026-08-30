//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability both Rust providers answer.
///
/// Named for what a caller gets — the items a file declares — rather than for how it is
/// obtained. `nomos.cap.syn.parse` would make the contract a description of one
/// implementation, and the second provider of the same capability could not honestly offer
/// it. That the second provider exists is what moved this constant here.
pub const CAPABILITY: &str = "nomos.cap.syntax.items";

/// The payload schema every answer to this capability is stamped with.
///
/// Versioned separately from the contract because the shape of the bytes and the meaning of
/// the question change for different reasons.
///
/// A schema is the shape of an answer and not a claim about its accuracy — that is what a
/// guarantee is for. Two providers of one capability writing different shapes would force
/// every consumer to know which one answered.
/// Versioned separately from the contract, and this is the version where that mattered.
/// v2 makes each item's documentation and declared shape an observation rather than a
/// string, so that a provider which cannot see one says so instead of writing the same
/// bytes as a provider that looked and found nothing. The *question* did not change, so
/// [`CONTRACT_VERSION`] did not either — see `OD-SYNTAX-002`.
pub const SCHEMA: &str = "nomos.syntax.items.v2";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::Syntactic`] is the ceiling because the capability is about what a file
/// says on its face. A compiler-backed provider that resolves names is answering a
/// different question and belongs behind a different contract; letting it offer this one at
/// [`FactVariant::SemanticallyResolved`] would mean two providers of one capability
/// disagreeing about what the capability means.
///
/// Completeness and granularity are deliberately *not* pinned to what any current provider
/// achieves. [`Assurance::Sound`] and [`IncrementalGranularity::Region`] leave room for a
/// provider that expands macros or reparses incrementally. A ceiling set to today's best
/// implementation has to be raised every time somebody improves something, and a ceiling
/// that moves is not a ceiling.
///
/// This is the clearest case for the crate. The ceiling binds every provider, and while it
/// lived in one of them, that one could raise or lower what its peer is permitted to claim.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Region,
    );
}

#[must_use]
pub fn Capability() -> CapabilityId
{
    return CapabilityId::New(CAPABILITY);
}

#[must_use]
pub fn Payload_Schema() -> SchemaId
{
    return SchemaId::New(SCHEMA);
}

/// The contract, to be declared once by whichever composition root builds a registry.
///
/// Declared by a root and not by a provider, which is the other half of the move. A
/// provider that declares the contract it offers against is asserting the terms of an
/// agreement it is a party to, and `Registry::Declare` would then refuse whichever provider
/// happened to be registered second.
#[must_use]
pub fn Capability_Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: CONTRACT_VERSION,
        summary: "The items a source file declares, as written, with the visibility each \
                  one declares and a count of the places the parse tree ends in unexpanded \
                  tokens."
            .to_owned(),
        ceiling: Ceiling(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_capability::contract_testing;

    /// The contract admits an answer weaker than its ceiling, which is what a ceiling is
    /// for. Without this the ceiling could be anything and nothing would notice until a
    /// provider tried to register.
    #[test]
    fn Test_The_Ceiling_Should_Admit_A_Weaker_Offer()
    {
        contract_testing::Assert_Ceiling_Admits(
            Capability_Contract(),
            Capability(),
            CONTRACT_VERSION,
            Guarantee::New(
                FactVariant::Approximate,
                Assurance::Unsound,
                Assurance::Unknown,
                IncrementalGranularity::File,
            ),
        );
    }

    /// And refuses one above it. This is the property the whole contract exists for, and it
    /// is asserted here — where the ceiling is written — rather than only in a provider,
    /// because no provider is entitled to be the place this is checked.
    #[test]
    fn Test_The_Ceiling_Should_Refuse_A_Claim_Of_Name_Resolution()
    {
        contract_testing::Assert_Ceiling_Refuses(
            Capability_Contract(),
            Guarantee::New(
                FactVariant::SemanticallyResolved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::Region,
            ),
            "a provider claiming resolution for a capability about what a file says on its \
             face would satisfy every rule that needs resolution",
        );
    }

    /// The contract stands without a provider.
    ///
    /// This is "removing either provider leaves the contract intact", stated as behaviour
    /// rather than as a diagram. This crate cannot name a provider — no dependency, and a
    /// band below both — so a registry built here holds the agreement and nobody to keep
    /// it, which resolves to coverage debt and not to a missing contract.
    #[test]
    fn Test_The_Contract_Should_Stand_With_No_Provider_At_All()
    {
        contract_testing::Assert_Stands_With_No_Provider(
            Capability_Contract(),
            Capability(),
            CONTRACT_VERSION,
            Ceiling(),
        );
    }

    /// One declaration per capability, which is the invariant that used to be doing this
    /// crate's job.
    #[test]
    fn Test_One_Capability_Should_Admit_One_Contract()
    {
        contract_testing::Assert_One_Contract_Per_Capability(
            Capability_Contract(),
            "one name, one meaning — and while this was the only thing keeping a second \
             provider from authoring its own terms, it was the registry compensating for \
             the layering",
        );
    }

    /// [`Payload_Schema`] is a leaf wrapper with no in-crate caller of its own — the four
    /// tests above drive [`Capability_Contract`], [`Capability`] and [`Ceiling`] but never
    /// this one, so it needs its own direct assertion rather than borrowing theirs.
    #[test]
    fn Test_Payload_Schema_Should_Match_The_Declared_Schema_Constant()
    {
        assert_eq!(Payload_Schema(), SchemaId::New(SCHEMA));
    }

    /// The other four tests each drive [`Capability_Contract`] only to reach a property of
    /// the ceiling or the registry; none asserts on the struct it built. This checks the
    /// construction itself: every field lands the value its own declaration says it should.
    #[test]
    fn Test_Capability_Contract_Should_Assemble_Its_Declared_Identity_And_Ceiling()
    {
        let contract = Capability_Contract();

        assert_eq!(contract.id, Capability());
        assert_eq!(contract.version, CONTRACT_VERSION);
        assert_eq!(contract.ceiling, Ceiling());
    }
}
