//! `nomos_cap_lint`'s own contract tests used to live inside `src/contract.rs`, compiled into
//! the crate itself — reaching `nomos_capability`'s `contract_testing` helpers and
//! `nomos_contracts`' types as an ordinary dependency that can still see this crate's own
//! private items, which is not the view a real consumer ever has. Compiled here instead, this
//! file can only reach `nomos_capability` and `nomos_contracts` through `nomos_cap_lint`'s own
//! public API — `Capability_Contract`, `Capability`, `Ceiling`, `CONTRACT_VERSION` and
//! `Payload_Schema`, all `pub` already — the same surface `nomos-rules` or a registry
//! composition root would use.

use nomos_cap_lint::{Capability, Capability_Contract, Ceiling, CONTRACT_VERSION, Payload_Schema, SCHEMA};
use nomos_capability::contract_testing;
use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, SchemaId};

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
/// asserting it watched the build actually execute, which reading a lint tool's own
/// static diagnostics does not do.
#[test]
fn Test_The_Ceiling_Should_Refuse_A_Claim_Of_Runtime_Observation()
{
    contract_testing::Assert_Ceiling_Refuses(
        Capability_Contract(),
        Guarantee::New(
            FactVariant::RuntimeObserved,
            Assurance::Sound,
            Assurance::Sound,
            IncrementalGranularity::Project,
        ),
        "a provider claiming runtime observation for a capability about a lint tool's \
         own static diagnostics would satisfy every rule that needs it",
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

/// [`Payload_Schema`] is a leaf wrapper with no other caller in this suite — the four
/// tests above drive [`Capability_Contract`], [`Capability`] and [`Ceiling`] but never
/// this one, so it needs its own direct assertion rather than borrowing theirs.
#[test]
fn Test_Payload_Schema_Should_Match_The_Declared_Schema_Constant()
{
    assert_eq!(Payload_Schema(), SchemaId::New(SCHEMA));
}

/// The other four tests each drive [`Capability_Contract`] only to reach a property of
/// the ceiling or the registry; none asserts on the struct it built. This checks the
/// construction itself: every field lands the value its own declaration says it should —
/// and, being a plain field read on the struct `nomos_capability::CapabilityContract`
/// hands back, it is also the most direct proof this suite is driving the real public
/// type and not a stand-in.
#[test]
fn Test_Capability_Contract_Should_Assemble_Its_Declared_Identity_And_Ceiling()
{
    let contract = Capability_Contract();

    assert_eq!(contract.id, Capability());
    assert_eq!(contract.version, CONTRACT_VERSION);
    assert_eq!(contract.ceiling, Ceiling());
}
