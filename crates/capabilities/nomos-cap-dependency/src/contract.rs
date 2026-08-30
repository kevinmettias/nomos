//! The agreement itself.

use nomos_capability::CapabilityContract;
use nomos_contracts::{
    Assurance, CapabilityId, ContractVersion, FactVariant, Guarantee, IncrementalGranularity,
    SchemaId,
};

/// The capability this crate answers.
///
/// Named for what a caller gets — a package's own first-party dependency edges — rather
/// than for how it is obtained, the same reason `nomos.cap.syntax.items` is named for its
/// answer rather than for `syn`.
pub const CAPABILITY: &str = "nomos.cap.dependency.edges";

/// The payload schema every answer to this capability is stamped with.
///
/// Versioned separately from the contract for the same reason `nomos-cap-syntax` gives:
/// the shape of the bytes and the meaning of the question change for different reasons.
pub const SCHEMA: &str = "nomos.dependency.edges.v1";

/// The contract version. Not a crate version: a caller reads against the contract.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::New(1, 0);

/// The strongest anything may claim for this capability.
///
/// [`FactVariant::SemanticallyResolved`] because a first-party dependency edge is not what
/// a `dependencies = {...}` block says on its face — a manifest can name a package by a
/// string that a text scan cannot tell apart from an unrelated crate of the same name on
/// a registry, the same ambiguity `OD-RULES-003` names for why this capability's answer
/// must come from Cargo's own resolution rather than from pattern-matching TOML. A
/// producer that only scanned manifest text belongs at `Syntactic`, not here.
///
/// [`IncrementalGranularity::Project`] because a package's dependency edges are declared
/// once for the whole package, in its manifest, and there is no file-level slice of that
/// declaration to refresh independently — the same reasoning `nomos.cap.module.index`
/// already gives for its own ceiling.
///
/// Completeness [`Assurance::Sound`], not pinned to what any current provider achieves:
/// this ceiling leaves room for a provider that also resolves generated or workspace-
/// inherited dependency declarations, the same way `nomos-cap-syntax`'s ceiling leaves
/// room for a provider that expands macros.
#[must_use]
pub const fn Ceiling() -> Guarantee
{
    return Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Project,
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
#[must_use]
pub fn Capability_Contract() -> CapabilityContract
{
    return CapabilityContract {
        id: Capability(),
        version: CONTRACT_VERSION,
        summary: "One package's own first-party dependency edges — every other workspace \
                  member it names as a dependency, each attributed the kind (normal, dev, \
                  build) and whether it is optional, as Cargo itself resolves the \
                  declaration rather than as a manifest's text alone can say."
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
    /// for.
    #[test]
    fn Test_The_Ceiling_Should_Admit_A_Weaker_Offer()
    {
        contract_testing::Assert_Ceiling_Admits(
            Capability_Contract(),
            Capability(),
            CONTRACT_VERSION,
            Guarantee::New(
                FactVariant::Syntactic,
                Assurance::Unsound,
                Assurance::Unknown,
                IncrementalGranularity::WholeWorkspace,
            ),
        );
    }

    /// And refuses one above it — a claim of `RuntimeObserved` would be a provider
    /// asserting it watched the build actually happen, which cargo's own manifest
    /// resolution does not do.
    #[test]
    fn Test_The_Ceiling_Should_Refuse_A_Claim_Above_Semantically_Resolved()
    {
        contract_testing::Assert_Ceiling_Refuses(
            Capability_Contract(),
            Guarantee::New(
                FactVariant::RuntimeObserved,
                Assurance::Sound,
                Assurance::Sound,
                IncrementalGranularity::Project,
            ),
            "a provider claiming runtime observation for a capability about declared \
             dependency edges would satisfy every rule that needs it",
        );
    }

    /// The contract stands without a provider.
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

    /// One declaration per capability.
    #[test]
    fn Test_One_Capability_Should_Admit_One_Contract()
    {
        contract_testing::Assert_One_Contract_Per_Capability(Capability_Contract(), "one name, one meaning");
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
