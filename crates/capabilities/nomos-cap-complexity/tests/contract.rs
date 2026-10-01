//! `nomos_cap_complexity`'s contract, tested from outside the crate through only what a consumer
//! can reach, the way `nomos_cap_controlflow`'s own `tests/contract.rs` is.

use nomos_cap_complexity::{Capability, Capability_Contract, Ceiling, CONTRACT_VERSION, Payload_Schema, SCHEMA};
use nomos_capability::contract_testing;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, SchemaId};

/// The contract admits an answer weaker than its ceiling: a count read from text is exactly the
/// first offer `OD-ROADMAP-004` says this family will see.
#[test]
fn Test_The_Ceiling_Should_Admit_A_Syntactic_Count()
{
    let guarantee = Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Unsound, IncrementalGranularity::File);

    contract_testing::Assert_Ceiling_Admits(Capability_Contract(), Capability(), CONTRACT_VERSION, guarantee);
}

/// And refuses one above it: a static count claiming to have watched the program run.
#[test]
fn Test_The_Ceiling_Should_Refuse_A_Claim_Above_Semantically_Resolved()
{
    let guarantee = Guarantee::New(FactVariant::RuntimeObserved, Assurance::Sound, Assurance::Sound, IncrementalGranularity::File);

    contract_testing::Assert_Ceiling_Refuses(
        Capability_Contract(),
        guarantee,
        "a provider claiming runtime observation for a count of static decision points would \
         satisfy every rule that needs it",
    );
}

#[test]
fn Test_The_Contract_Should_Stand_With_No_Provider_At_All()
{
    contract_testing::Assert_Stands_With_No_Provider(Capability_Contract(), Capability(), CONTRACT_VERSION, Ceiling());
}

#[test]
fn Test_One_Capability_Should_Admit_One_Contract()
{
    contract_testing::Assert_One_Contract_Per_Capability(Capability_Contract(), "one name, one meaning");
}

#[test]
fn Test_Payload_Schema_Should_Match_The_Declared_Schema_Constant()
{
    assert_eq!(Payload_Schema(), SchemaId::New(SCHEMA));
}

#[test]
fn Test_Capability_Contract_Should_Assemble_Its_Declared_Identity_And_Ceiling()
{
    let contract = Capability_Contract();

    assert_eq!(contract.id, Capability());
    assert_eq!(contract.version, CONTRACT_VERSION);
    assert_eq!(contract.ceiling, Ceiling());
}
