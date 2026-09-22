//! What the shared currency check promises, exercised over a fact built by hand.
//!
//! By hand rather than through a provider, because what is under test here is the digest
//! itself: which differences between two facts it is sensitive to. A provider produces one
//! fact per call and cannot be asked for two that differ in exactly one component, which is
//! what every row of
//! [`Test_A_Fact_Differing_In_Any_Observable_Component_Should_Be_Written`] needs.
//! `crate::tests::currency` is where each real family's own materializer is driven through
//! this check over real inputs.

use super::{Is_Already_Current, Materialized_Or_Already_Current};

use nomos_analysis::{Context, FactKey, FactPayload, GuaranteeDigest, InputDigest, MaterializedFact, MemoryFactStore};
use nomos_contracts::{
    Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128,
    EvidenceClass, FactVariant, GenerationId, Guarantee, IncrementalGranularity, ProviderId,
    SchemaId, SnapshotId, SubjectId,
};

/// Fill bytes distinct enough that the digests below differ from one another; each value
/// carries no meaning beyond "not equal to the others" -- the convention
/// `nomos_analysis`'s own store tests already use for the same fixtures.
const SUBJECT_FILL: u8 = 1;
const VARIANT_FILL: u8 = 3;
const CONFIGURATION_FILL: u8 = 4;
const SNAPSHOT_FILL: u8 = 2;

/// The generation every fixture here is filed and read at. One value, because the whole
/// point of the check is what happens when a family is re-materialized at a generation the
/// store has not left -- which is exactly the real case for a policy or manifest input that
/// moved while no source in `sources` did, since only a source edit advances
/// `nomos_workspace::Workspace`'s own generation.
const FIXTURE_GENERATION: u64 = 4;

/// The schema every fixture fact but the schema-mutated row declares.
const FIXTURE_SCHEMA: &str = "nomos.test.check_orchestration.currency.v1";

/// A second, distinct schema, for the row that proves the digest reads the schema and not
/// only the bytes beside it.
const OTHER_FIXTURE_SCHEMA: &str = "nomos.test.check_orchestration.currency.v2";

fn Seeded_Digest(fill: u8) -> Digest128
{
    return Digest128::From_Bytes([fill; Digest128::BYTE_LENGTH]);
}

fn Fixture_Context() -> Context
{
    return Context {
        snapshot: SnapshotId::From_Digest(Seeded_Digest(SNAPSHOT_FILL)),
        variant: BuildVariantId::From_Digest(Seeded_Digest(VARIANT_FILL)),
        configuration: ConfigurationId::From_Digest(Seeded_Digest(CONFIGURATION_FILL)),
        generation: GenerationId::From_Raw(FIXTURE_GENERATION),
    };
}

fn Fixture_Guarantee() -> Guarantee
{
    return Guarantee::New(FactVariant::Syntactic, Assurance::Sound, Assurance::Sound, IncrementalGranularity::WholeWorkspace);
}

/// The key every non-syntax provider in this workspace files under: an **empty**
/// `semantic_inputs`, which is why a key-only currency check cannot tell a changed input
/// from an unchanged one for any of them, and why
/// [`Test_A_Fact_Differing_In_Any_Observable_Component_Should_Be_Written`]'s payload row is
/// the load-bearing one.
fn Fixture_Key() -> FactKey
{
    return FactKey {
        contract: CapabilityId::New("nomos.cap.test.currency"),
        contract_version: ContractVersion::New(1, 0),
        subject: SubjectId::From_Digest(Seeded_Digest(SUBJECT_FILL)),
        semantic_inputs: InputDigest::Of(&[]),
        provider: ProviderId::New("nomos.provider.test.currency"),
        provider_version: ContractVersion::New(1, 0),
        guarantee: GuaranteeDigest::Of(&Fixture_Guarantee()),
        variant: BuildVariantId::From_Digest(Seeded_Digest(VARIANT_FILL)),
        configuration: ConfigurationId::From_Digest(Seeded_Digest(CONFIGURATION_FILL)),
    };
}

fn Fixture_Fact(key: FactKey, payload: FactPayload, evidence: EvidenceClass) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.At(GenerationId::From_Raw(FIXTURE_GENERATION)),
        snapshot: SnapshotId::From_Digest(Seeded_Digest(SNAPSHOT_FILL)),
        evidence,
        guarantee: Fixture_Guarantee(),
        payload,
    };
}

fn Fixture_Payload(bytes: &[u8]) -> FactPayload
{
    return FactPayload::New(SchemaId::New(FIXTURE_SCHEMA), bytes.to_vec());
}

/// The fact every test below starts from.
fn Original_Fact() -> MaterializedFact
{
    return Fixture_Fact(Fixture_Key(), Fixture_Payload(b"declared"), EvidenceClass::Derived);
}

/// A store already serving [`Original_Fact`].
fn Store_Serving_The_Original() -> MemoryFactStore
{
    let mut store = MemoryFactStore::New();
    let written = Materialized_Or_Already_Current(Original_Fact(), &Fixture_Context(), &mut store);

    assert!(written, "a fresh store cannot refuse the first write of a fact");
    assert_eq!(store.Materializations(), 1, "the first write of a fact a store does not hold must be a real write");

    return store;
}

/// An empty store holds nothing, so nothing is current in it -- the case every materializer
/// reaches on its first call, and the one that must fall through to a real write.
#[test]
fn Test_Is_Already_Current_Should_Be_False_For_A_Fact_The_Store_Does_Not_Hold()
{
    let store = MemoryFactStore::New();

    assert!(
        !Is_Already_Current(&Original_Fact(), &Fixture_Context(), &store),
        "an empty store serves nothing, so no candidate can be already current in it"
    );
}

/// The skip itself: a second, byte-identical materialization of a fact the store is already
/// serving must not be filed again.
///
/// The store's own counter is the observable rather than the return value, for the reason
/// [`Materialized_Or_Already_Current`]'s own doc gives: it answers coverage, so it is `true`
/// for a skip and for a write alike and cannot tell them apart.
#[test]
fn Test_Materialized_Or_Already_Current_Should_Skip_A_Byte_Identical_Rewrite()
{
    let mut store = Store_Serving_The_Original();

    let current = Materialized_Or_Already_Current(Original_Fact(), &Fixture_Context(), &mut store);

    assert!(current, "the subject still has a current fact after the skip, which is what the caller counts");
    assert_eq!(
        store.Materializations(), 1,
        "an identical fact was filed a second time; the store now holds two copies of one observation and \
         `Materialization_Tracking` reads the counter's move as the family having changed"
    );
}

/// One way a candidate can differ from what the store serves, and the name a failure quotes.
struct MutatedFact
{
    component: &'static str,
    fact: MaterializedFact,
}

/// A candidate differing from [`Original_Fact`] in exactly one observable component, one row
/// per component [`super::Input_Digest_Of`] reads.
///
/// The payload row is the one that decides whether this check is honest at all. Every
/// non-syntax provider in this workspace files with an empty `semantic_inputs`, so a changed
/// policy file, a changed manifest and a changed clippy run all produce a fact under the
/// *same* key as the one before them. A check that compared keys alone would skip every one
/// of those writes and serve a stale fact, and it would pass every other row here.
fn Mutated_Facts() -> Vec<MutatedFact>
{
    let mut key_elsewhere = Fixture_Key();
    key_elsewhere.semantic_inputs = InputDigest::Of(&[b"a different reading"]);

    return vec![
        MutatedFact { component: "payload bytes", fact: Fixture_Fact(Fixture_Key(), Fixture_Payload(b"redeclared"), EvidenceClass::Derived) },
        MutatedFact {
            component: "payload schema",
            fact: Fixture_Fact(Fixture_Key(), FactPayload::New(SchemaId::New(OTHER_FIXTURE_SCHEMA), b"declared".to_vec()), EvidenceClass::Derived),
        },
        MutatedFact { component: "evidence class", fact: Fixture_Fact(Fixture_Key(), Fixture_Payload(b"declared"), EvidenceClass::Observed) },
        MutatedFact { component: "fact key", fact: Fixture_Fact(key_elsewhere, Fixture_Payload(b"declared"), EvidenceClass::Derived) },
    ];
}

/// The guard's falsifier, one per component the digest reads: a candidate differing from
/// what the store serves in any of them is a real write, not a skip.
#[test]
fn Test_A_Fact_Differing_In_Any_Observable_Component_Should_Be_Written()
{
    for mutated in Mutated_Facts()
    {
        let mut store = Store_Serving_The_Original();

        let current = Materialized_Or_Already_Current(mutated.fact, &Fixture_Context(), &mut store);

        assert!(current, "{}: a write the store accepts leaves the subject with a current fact", mutated.component);
        assert_eq!(
            store.Materializations(), 2,
            "{}: a fact differing from what the store serves was skipped as already current, so a reused store \
             now answers with inputs that have moved",
            mutated.component
        );
    }
}
