//! This capability's own contract tests, compiled outside the crate they judge.
//!
//! Outside rather than inside `src/contract.rs`, the same division
//! `nomos-cap-dependency-policy`'s own `tests/contract.rs` draws and for the same reason: a
//! test compiled into the crate reaches `nomos_capability`'s `contract_testing` helpers as
//! an ordinary dependency that can still see this crate's private items, which is not the
//! view a real consumer ever has. Compiled here, this file reaches the contract only through
//! the public surface `nomos-lang-rust-compiler` and `nomos-rules` reach it through.

use nomos_cap_rust_copy_clones::{
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
        IncrementalGranularity::Project,
    );

    contract_testing::Assert_Ceiling_Admits(
        Capability_Contract(),
        Capability(),
        CONTRACT_VERSION,
        guarantee,
    );
}

/// And refuses one above it -- a claim of `RuntimeObserved` would be a provider asserting it
/// watched a real execution clone a `Copy` value, which resolving a call's type statically
/// does not do.
#[test]
fn Test_The_Ceiling_Should_Refuse_A_Claim_Of_Runtime_Observation()
{
    let guarantee = Guarantee::New(
        FactVariant::RuntimeObserved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::Project,
    );

    contract_testing::Assert_Ceiling_Refuses(
        Capability_Contract(),
        guarantee,
        "a provider claiming runtime observation for a capability about a statically \
         resolved call type would satisfy every rule that needs it",
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

/// [`Payload_Schema`] is a leaf wrapper with no other caller in this suite, so it needs its
/// own direct assertion rather than borrowing one of the four above.
#[test]
fn Test_Payload_Schema_Should_Match_The_Declared_Schema_Constant()
{
    assert_eq!(Payload_Schema(), SchemaId::New(SCHEMA));
}

/// Every field of the assembled contract lands the value its own declaration says it should
/// -- and, being a plain field read on the struct `nomos_capability::CapabilityContract`
/// hands back, this is also the most direct proof the suite drives the real public type.
#[test]
fn Test_Capability_Contract_Should_Assemble_Its_Declared_Identity_And_Ceiling()
{
    let contract = Capability_Contract();

    assert_eq!(contract.id, Capability());
    assert_eq!(contract.version, CONTRACT_VERSION);
    assert_eq!(contract.ceiling, Ceiling());
}
