//! `nomos_cap_words_policy`'s own contract tests, compiled here rather than inside
//! `src/contract.rs` for the same reason its three siblings' are: this file can only reach
//! `nomos_capability` and `nomos_contracts` through `nomos_cap_words_policy`'s own public
//! API — `Capability_Contract`, `Capability`, `Ceiling`, `CONTRACT_VERSION` and
//! `Payload_Schema`, all `pub` already — the same surface `nomos-rules` or a registry
//! composition root would use.

use nomos_cap_words_policy::{
    Capability, Capability_Contract, Ceiling, CONTRACT_VERSION, Payload_Schema, SCHEMA,
};
use nomos_capability::contract_testing;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, SchemaId};

/// The contract admits an answer weaker than its ceiling, which is what a ceiling is for.
#[test]
fn Test_The_Ceiling_Should_Admit_A_Weaker_Offer()
{
    let guarantee = Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Unsound,
        Assurance::Unknown,
        IncrementalGranularity::WholeWorkspace,
    );

    contract_testing::Assert_Ceiling_Admits(
        Capability_Contract(),
        Capability(),
        CONTRACT_VERSION,
        guarantee,
    );
}

/// And refuses one above it — a claim of `SemanticallyResolved` would be a provider
/// asserting it resolved something, when reading a repository's own declared
/// configuration resolves nothing.
#[test]
fn Test_The_Ceiling_Should_Refuse_A_Claim_Of_Semantic_Resolution()
{
    let guarantee = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::WholeWorkspace,
    );

    contract_testing::Assert_Ceiling_Refuses(
        Capability_Contract(),
        guarantee,
        "a provider claiming semantic resolution for a capability about a repository's own \
         declared configuration would satisfy every rule that needs it",
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

/// [`Payload_Schema`] is a leaf wrapper with no other caller in this suite.
#[test]
fn Test_Payload_Schema_Should_Match_The_Declared_Schema_Constant()
{
    assert_eq!(Payload_Schema(), SchemaId::New(SCHEMA));
}

/// The other four tests each drive [`Capability_Contract`] only to reach a property of
/// the ceiling or the registry; this checks the construction itself.
#[test]
fn Test_Capability_Contract_Should_Assemble_Its_Declared_Identity_And_Ceiling()
{
    let contract = Capability_Contract();

    assert_eq!(contract.id, Capability());
    assert_eq!(contract.version, CONTRACT_VERSION);
    assert_eq!(contract.ceiling, Ceiling());
}
