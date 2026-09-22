//! What a store written by one process means to another that reads it.
//!
//! The fixture is the one `tests/recomputation_equivalence.rs` uses, and deliberately: that
//! file states the property that matters once anything is rematerialized — recomputing
//! incrementally after a change and recomputing the same input state from an empty store
//! must produce the same facts — over a graph with one real derived hop, whose downstream
//! *payload* is a function of the upstream payload, so a stale downstream value is
//! observably different from a fresh one. The two proofs here are that property asked of a
//! store that crossed a process boundary: once over an unchanged tree, and once across an
//! edit. A fixture where both computations trivially agreed would assert nothing, so the
//! control below holds the same line its counterpart there holds.
//!
//! These live in the crate rather than in `tests/`, because this item's verification
//! predicate runs the library's own tests and a proof it does not run is not a proof it has.

use std::path::Path;
use std::path::PathBuf;

use nomos_contracts::{
    Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128,
    EvidenceClass, FactVariant, GenerationId, Guarantee, IncrementalGranularity, ProviderId,
    SchemaId, SnapshotId, SubjectId,
};

use crate::{
    Dependency, FactKey, FactPayload, FactStore, GenerationCause, GuaranteeDigest, InputDigest,
    InvalidationReport, MaterializedFact, MemoryFactStore, PersistenceError, ReadOutcome,
};

use super::Store_File;

/// The components every fixture key shares: one variant, one configuration and one snapshot
/// throughout, so the two keys differ only in the subject they name.
const FIXTURE_SNAPSHOT_SEED: u8 = 2;
const FIXTURE_VARIANT_SEED: u8 = 3;
const FIXTURE_CONFIGURATION_SEED: u8 = 4;

/// The derived fact's subject: its own, not the leaf's, so the only thing that reaches it is
/// the dependency edge the downstream fact records.
const DOWNSTREAM_SUBJECT_SEED: u8 = 2;

/// The two payload schemas this fixture writes, which a reading build must declare it
/// understands before a store carrying them is served.
const LEAF_SCHEMA: &str = "nomos.fixture.leaf.v1";
const ROLLUP_SCHEMA: &str = "nomos.fixture.rollup.v1";

/// The leaf payload before and after the edit. They differ in content *and* in length, so a
/// rollup of one is never a rollup of the other.
const INITIAL_PAYLOAD: &[u8] = b"alpha";
const CHANGED_PAYLOAD: &[u8] = b"omega-longer";

/// Whether the downstream fact is materialized with the dependency edge that makes it
/// followable, or without it.
#[derive(Clone, Copy)]
enum DependencyEdge
{
    Recorded,
    Omitted,
}

/// The two facts this fixture is about: the leaf, and the derived fact that reads it.
struct FixtureFacts
{
    upstream: MaterializedFact,
    downstream: MaterializedFact,
}

#[test]
fn Test_A_Reloaded_Store_Over_An_Unchanged_Tree_Should_Agree_With_A_Clean_Rebuild()
{
    let directory = Temporary_Directory("unchanged");
    let mut written = MemoryFactStore::New();
    Materialize_Pair(&mut written, INITIAL_PAYLOAD, GenerationId::INITIAL, &Recorded_Edge());
    Write_Store(&written, &directory);

    let mut reloaded = Reloaded(&directory);
    // The run over an unchanged tree: nothing has been invalidated, and the same facts are
    // materialized again at the same generation, exactly as a second invocation would.
    Materialize_Pair(&mut reloaded, INITIAL_PAYLOAD, GenerationId::INITIAL, &Recorded_Edge());

    let served = Current_Pair(&reloaded, GenerationId::INITIAL);
    let clean = Recompute_From_Empty(INITIAL_PAYLOAD, GenerationId::INITIAL);

    assert_eq!(served.upstream, clean.upstream, "the reloaded leaf disagrees with a clean rebuild");
    assert_eq!(
        served.downstream, clean.downstream,
        "the reloaded derived fact disagrees with a clean rebuild"
    );
}

/// Two stores holding the same facts write the same bytes, whatever order they were shown
/// them in.
///
/// The running store addresses a fact by a slot, which is a position in one process's
/// interning table; the file addresses it by digest, because a slot means nothing to the
/// next process. Nothing makes one process observe its facts in the order another did — a
/// check run picks up subjects in whatever order its walk reaches them — so a written form
/// that carried interning order would differ between two stores that hold exactly the same
/// thing, and two processes could not compare their stores at all.
///
/// The two orders below are chosen so they *do* differ: the digests decide the written order
/// and the control below asserts that the two observation orders disagree with it, because a
/// fixture whose two orders happened to coincide would assert nothing. Both ends of the
/// dependents graph are covered too — it is an ordered map keyed on slots, which is the
/// structure most likely to leak that order.
#[test]
fn Test_Two_Stores_Holding_The_Same_Facts_Should_Write_The_Same_Bytes()
{
    let observed_upstream_first = Temporary_Directory("deterministic-upstream-first");
    let observed_downstream_first = Temporary_Directory("deterministic-downstream-first");

    let mut upstream_first = MemoryFactStore::New();
    Materialize_Pair(&mut upstream_first, INITIAL_PAYLOAD, GenerationId::INITIAL, &Recorded_Edge());
    Write_Store(&upstream_first, &observed_upstream_first);

    let mut downstream_first = MemoryFactStore::New();
    downstream_first
        .Materialize(Downstream_Fact(INITIAL_PAYLOAD, GenerationId::INITIAL), &Recorded_Edge())
        .expect("a fresh store holds nothing to conflict with");
    downstream_first
        .Materialize(Upstream_Fact(INITIAL_PAYLOAD, GenerationId::INITIAL), &[])
        .expect("the upstream key has not been written yet");
    Write_Store(&downstream_first, &observed_downstream_first);

    Assert_Wrote_The_Same_Bytes(&observed_upstream_first, &observed_downstream_first);
}

/// The control the assertion above needs: the two observation orders really are different
/// orders, so the comparison is between two files that had something to disagree about.
#[test]
fn Test_The_Two_Fixture_Keys_Should_Hash_To_Different_Digests()
{
    assert_ne!(
        Upstream_Key().Digest(),
        Downstream_Key().Digest(),
        "the two fixture keys hash to one digest, so the two observation orders above are          the same order and the comparison between them asserts nothing"
    );
}

fn Assert_Wrote_The_Same_Bytes(first: &Path, second: &Path)
{
    let before = std::fs::read(Store_File(first)).expect("the store just written is readable");
    let after = std::fs::read(Store_File(second)).expect("the store just written is readable");

    assert!(before.len() > 1, "the fixture wrote nothing, so this compares two empty files");
    assert_eq!(
        String::from_utf8_lossy(&after),
        String::from_utf8_lossy(&before),
        "two stores holding the same facts wrote different bytes, so the written form \
         carries the order one process was shown them in rather than the order its own \
         contents have"
    );
}

/// A store written, reloaded and written again writes the same bytes.
///
/// The narrower half of the same question, and the one a host actually performs: a reload
/// interns in the file's own order, and writing that store back must not shift anything.
#[test]
fn Test_A_Store_Written_Reloaded_And_Written_Again_Should_Write_The_Same_Bytes()
{
    let first = Temporary_Directory("deterministic-first");
    let second = Temporary_Directory("deterministic-second");
    let mut written = MemoryFactStore::New();
    Materialize_Pair(&mut written, INITIAL_PAYLOAD, GenerationId::INITIAL, &Recorded_Edge());
    Write_Store(&written, &first);

    Write_Store(&Reloaded(&first), &second);

    Assert_Wrote_The_Same_Bytes(&first, &second);
}

#[test]
fn Test_A_Reloaded_Store_Should_Invalidate_Exactly_What_The_In_Memory_Store_Would()
{
    let directory = Temporary_Directory("edit");
    let mut in_memory = MemoryFactStore::New();
    Materialize_Pair(&mut in_memory, INITIAL_PAYLOAD, GenerationId::INITIAL, &Recorded_Edge());
    Write_Store(&in_memory, &directory);
    let mut reloaded = Reloaded(&directory);

    let after_edit = GenerationId::INITIAL.Next();
    let from_memory = Invalidate_For_The_Edit(&mut in_memory, after_edit);
    let from_disk = Invalidate_For_The_Edit(&mut reloaded, after_edit);

    assert!(
        !from_memory.dependent.is_empty(),
        "the in-memory store followed no dependency edge, so this compares two empty answers"
    );
    assert_eq!(
        from_disk, from_memory,
        "the reloaded store did not invalidate what the store that wrote it would have"
    );
}

#[test]
fn Test_A_Reloaded_Store_Rematerialized_After_An_Edit_Should_Agree_With_A_Clean_Rebuild()
{
    let directory = Temporary_Directory("rematerialized");
    let mut written = MemoryFactStore::New();
    Materialize_Pair(&mut written, INITIAL_PAYLOAD, GenerationId::INITIAL, &Recorded_Edge());
    Write_Store(&written, &directory);
    let mut reloaded = Reloaded(&directory);

    let after_edit = GenerationId::INITIAL.Next();
    let named = Invalidate_For_The_Edit(&mut reloaded, after_edit);
    Rematerialize_Named(&mut reloaded, &Named_In(&named), CHANGED_PAYLOAD, &Recorded_Edge());

    let served = Current_Pair(&reloaded, after_edit);
    let clean = Recompute_From_Empty(CHANGED_PAYLOAD, after_edit);

    assert_eq!(served.upstream, clean.upstream, "the reloaded leaf disagrees after the edit");
    assert_eq!(
        served.downstream, clean.downstream,
        "the reloaded derived fact is stale - this is what a store whose edges did not \
         survive the process boundary serves: a stale answer that looks correct"
    );
}

/// Proves the two tests above are not vacuous. A downstream fact materialized with no
/// dependency edge is never named in `report.dependent`, is never rematerialized, and its
/// stale value survives the round trip and the read undetected. If this ever stops
/// disagreeing, the fixture can no longer tell a store that carried its edges across the
/// process boundary from one that lost them.
#[test]
fn Test_A_Reloaded_Store_Whose_Edge_Was_Never_Recorded_Should_Serve_A_Stale_Value()
{
    let directory = Temporary_Directory("unrecorded");
    let mut written = MemoryFactStore::New();
    Materialize_Pair(&mut written, INITIAL_PAYLOAD, GenerationId::INITIAL, &Omitted_Edge());
    Write_Store(&written, &directory);
    let mut reloaded = Reloaded(&directory);

    let after_edit = GenerationId::INITIAL.Next();
    let named = Invalidate_For_The_Edit(&mut reloaded, after_edit);
    Rematerialize_Named(&mut reloaded, &Named_In(&named), CHANGED_PAYLOAD, &Omitted_Edge());

    let served = Current_Pair(&reloaded, after_edit);
    let clean = Recompute_From_Empty(CHANGED_PAYLOAD, after_edit);

    assert_ne!(
        served.downstream.payload.bytes, clean.downstream.payload.bytes,
        "an unrecorded edge should have left a stale downstream value"
    );
}

#[test]
fn Test_A_Round_Trip_Should_Keep_The_Write_Count_And_Every_Recorded_Dependency()
{
    let directory = Temporary_Directory("round-trip");
    let mut written = MemoryFactStore::New();
    Materialize_Pair(&mut written, INITIAL_PAYLOAD, GenerationId::INITIAL, &Recorded_Edge());
    Write_Store(&written, &directory);

    let reloaded = Reloaded(&directory);

    assert_eq!(reloaded.Materializations(), written.Materializations());
    assert_eq!(reloaded.Live(), written.Live());
    assert_eq!(reloaded.Dependencies_Of(&Downstream_Key()), Recorded_Edge());
}

#[test]
fn Test_Read_Should_Refuse_A_Truncated_File_And_Name_It()
{
    let directory = Temporary_Directory("truncated");
    Write_Store(&Store_With_One_Pair(), &directory);
    let file = Store_File(&directory);
    let whole = std::fs::read(&file).expect("the store this test just wrote is readable");
    let kept = whole.len().saturating_div(2);
    std::fs::write(&file, whole.get(..kept).expect("half of a file is inside it"))
        .expect("the file this test just read is writable");

    let refused = Refusal_Of(&directory);

    assert!(matches!(refused, PersistenceError::Corrupt { .. }), "{refused:?} is not a corruption");
    assert_eq!(refused.File(), file);
}

#[test]
fn Test_Read_Should_Refuse_A_File_Written_Under_Another_Key_Shape_And_Name_It()
{
    let directory = Temporary_Directory("key-shape");
    Write_Store(&Store_With_One_Pair(), &directory);
    Rewrite_Store_File(&directory, "\"key_shape\":[\"contract\"", "\"key_shape\":[\"snapshot\"");

    let refused = Refusal_Of(&directory);

    assert!(
        matches!(refused, PersistenceError::ForeignKeyShape { .. }),
        "{refused:?} is not a key-shape refusal"
    );
    assert_eq!(refused.File(), Store_File(&directory));
}

#[test]
fn Test_Read_Should_Refuse_A_File_Written_Under_Another_Format_Version_And_Name_It()
{
    let directory = Temporary_Directory("format");
    Write_Store(&Store_With_One_Pair(), &directory);
    Rewrite_Store_File(&directory, "\"format_version\":1", "\"format_version\":2");

    let refused = Refusal_Of(&directory);

    assert!(
        matches!(refused, PersistenceError::ForeignFormat { .. }),
        "{refused:?} is not a format refusal"
    );
    assert_eq!(refused.File(), Store_File(&directory));
}

#[test]
fn Test_Read_Should_Refuse_A_Payload_Schema_The_Reading_Build_Does_Not_Understand()
{
    let directory = Temporary_Directory("schema");
    Write_Store(&Store_With_One_Pair(), &directory);

    // The build understands the leaf and nothing else, which is what a build composed with
    // one provider of two looks like.
    let refused = MemoryFactStore::Read_From_Directory(&directory, &[SchemaId::New(LEAF_SCHEMA)])
        .err()
        .expect("a store carrying an undeclared schema must not be served");

    assert!(
        matches!(refused, PersistenceError::UnknownPayloadSchema { .. }),
        "{refused:?} is not a schema refusal"
    );
    assert_eq!(refused.File(), Store_File(&directory));
}

/// Every schema this fixture's facts are written under. A real host declares what its own
/// composed providers emit; the fixture declares its own two.
fn Understood_Schemas() -> Vec<SchemaId>
{
    return vec![SchemaId::New(LEAF_SCHEMA), SchemaId::New(ROLLUP_SCHEMA)];
}

fn Write_Store(store: &MemoryFactStore, directory: &Path)
{
    store.Write_To_Directory(directory).expect("a fresh temporary directory is writable");
}

fn Reloaded(directory: &Path) -> MemoryFactStore
{
    return MemoryFactStore::Read_From_Directory(directory, &Understood_Schemas())
        .expect("a store this build wrote, under this build's own key shape, is readable back");
}

fn Refusal_Of(directory: &Path) -> PersistenceError
{
    return MemoryFactStore::Read_From_Directory(directory, &Understood_Schemas())
        .err()
        .expect("the file was damaged or made foreign before this read");
}

fn Rewrite_Store_File(directory: &Path, from: &str, into: &str)
{
    let file = Store_File(directory);
    let written = std::fs::read_to_string(&file).expect("the store this test just wrote is readable");
    assert!(written.contains(from), "the written form no longer spells `{from}`");
    std::fs::write(&file, written.replace(from, into))
        .expect("the file this test just read is writable");
}

fn Store_With_One_Pair() -> MemoryFactStore
{
    let mut store = MemoryFactStore::New();
    Materialize_Pair(&mut store, INITIAL_PAYLOAD, GenerationId::INITIAL, &Recorded_Edge());

    return store;
}

fn Temporary_Directory(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-analysis-persistence-{name}-{}", std::process::id()));
    if path.exists()
    {
        std::fs::remove_dir_all(&path).expect("the previous run's synthetic directory is removable");
    }
    std::fs::create_dir_all(&path).expect("test needs a temporary directory");

    return path;
}

fn Recorded_Edge() -> Vec<Dependency>
{
    return Edges(DependencyEdge::Recorded);
}

fn Omitted_Edge() -> Vec<Dependency>
{
    return Edges(DependencyEdge::Omitted);
}

fn Edges(edge: DependencyEdge) -> Vec<Dependency>
{
    return match edge
    {
        DependencyEdge::Recorded => vec![Downstream_Dependency()],
        DependencyEdge::Omitted => Vec::new(),
    };
}

/// Invalidates the upstream subject, which is what an edit to the leaf's file is to a store.
fn Invalidate_For_The_Edit(store: &mut MemoryFactStore, at: GenerationId) -> InvalidationReport
{
    return store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: Upstream_Key().subject,
            granularity: IncrementalGranularity::File,
        },
        at,
    );
}

/// Every key a report named, direct and dependent together, in the order a real caller would
/// rematerialize them.
fn Named_In(report: &InvalidationReport) -> Vec<FactKey>
{
    let mut named = report.direct.clone();
    named.extend(report.dependent.clone());

    return named;
}

fn Materialize_Pair(
    store: &mut MemoryFactStore,
    payload: &[u8],
    generation: GenerationId,
    edges: &[Dependency],
)
{
    store.Materialize(Upstream_Fact(payload, generation), &[]).expect("materializes upstream");
    store
        .Materialize(Downstream_Fact(payload, generation), edges)
        .expect("materializes downstream");
}

fn Rematerialize_Named(
    store: &mut MemoryFactStore,
    named: &[FactKey],
    payload: &[u8],
    edges: &[Dependency],
)
{
    let at = GenerationId::INITIAL.Next();

    for key in named
    {
        if *key == Upstream_Key()
        {
            store.Materialize(Upstream_Fact(payload, at), &[]).expect("rematerializes upstream");
        }
        else if *key == Downstream_Key()
        {
            store
                .Materialize(Downstream_Fact(payload, at), edges)
                .expect("rematerializes downstream");
        }
    }
}

fn Current_Pair(store: &MemoryFactStore, at: GenerationId) -> FixtureFacts
{
    return FixtureFacts {
        upstream: store
            .Current(&Upstream_Key().At(at), at)
            .expect("the leaf must be readable after a recompute"),
        downstream: store
            .Current(&Downstream_Key().At(at), at)
            .expect("the derived fact must be readable after a recompute"),
    };
}

fn Recompute_From_Empty(payload: &[u8], at: GenerationId) -> FixtureFacts
{
    let mut store = MemoryFactStore::New();
    Materialize_Pair(&mut store, payload, at, &Recorded_Edge());

    return Current_Pair(&store, at);
}

/// A real recomputation, not a stand-in: the derived fact's payload is the upstream bytes
/// reversed, so changing the upstream bytes changes this output.
fn Rollup_Of(payload: &[u8]) -> Vec<u8>
{
    let mut reversed = payload.to_vec();
    reversed.reverse();

    return reversed;
}

fn Upstream_Fact(payload: &[u8], generation: GenerationId) -> MaterializedFact
{
    return MaterializedFact {
        identity: Upstream_Key().At(generation),
        snapshot: SnapshotId::From_Digest(Seeded_Digest(FIXTURE_SNAPSHOT_SEED)),
        evidence: EvidenceClass::Verified,
        guarantee: Fixture_Guarantee(),
        payload: FactPayload::New(SchemaId::New(LEAF_SCHEMA), payload.to_vec()),
    };
}

fn Downstream_Fact(payload: &[u8], generation: GenerationId) -> MaterializedFact
{
    return MaterializedFact {
        identity: Downstream_Key().At(generation),
        snapshot: SnapshotId::From_Digest(Seeded_Digest(FIXTURE_SNAPSHOT_SEED)),
        evidence: EvidenceClass::Derived,
        guarantee: Fixture_Guarantee(),
        payload: FactPayload::New(SchemaId::New(ROLLUP_SCHEMA), Rollup_Of(payload)),
    };
}

fn Downstream_Dependency() -> Dependency
{
    return Dependency {
        key: Upstream_Key(),
        outcome: ReadOutcome::Materialized,
    };
}

fn Upstream_Key() -> FactKey
{
    return FactKey {
        contract: CapabilityId::New("nomos.cap.fixture.leaf"),
        contract_version: ContractVersion::New(1, 0),
        subject: SubjectId::From_Digest(Seeded_Digest(1)),
        semantic_inputs: InputDigest::Of(&[b"fixture"]),
        provider: ProviderId::New("nomos.provider.fixture.leaf"),
        provider_version: ContractVersion::New(1, 0),
        guarantee: GuaranteeDigest::Of(&Fixture_Guarantee()),
        variant: BuildVariantId::From_Digest(Seeded_Digest(FIXTURE_VARIANT_SEED)),
        configuration: ConfigurationId::From_Digest(Seeded_Digest(FIXTURE_CONFIGURATION_SEED)),
    };
}

fn Downstream_Key() -> FactKey
{
    return FactKey {
        contract: CapabilityId::New("nomos.cap.fixture.rollup"),
        contract_version: ContractVersion::New(1, 0),
        subject: SubjectId::From_Digest(Seeded_Digest(DOWNSTREAM_SUBJECT_SEED)),
        semantic_inputs: InputDigest::Of(&[b"fixture-rollup"]),
        provider: ProviderId::New("nomos.provider.fixture.rollup"),
        provider_version: ContractVersion::New(1, 0),
        guarantee: GuaranteeDigest::Of(&Fixture_Guarantee()),
        variant: BuildVariantId::From_Digest(Seeded_Digest(FIXTURE_VARIANT_SEED)),
        configuration: ConfigurationId::From_Digest(Seeded_Digest(FIXTURE_CONFIGURATION_SEED)),
    };
}

fn Fixture_Guarantee() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::File,
    );
}

fn Seeded_Digest(seed: u8) -> Digest128
{
    return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
}
