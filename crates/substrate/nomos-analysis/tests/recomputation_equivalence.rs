//! `Condensation_Of` (`invalidation_order.rs`) establishes that a caller can recover a valid
//! rematerialization order. It says nothing about whether following that order actually
//! produces the right answer. This file asserts the property that matters once anything is
//! rematerialized: recomputing incrementally after a change, and recomputing the same input
//! state from an empty store, must produce the same facts.
//!
//! The graph has one real derived hop - a downstream fact whose *payload*, not merely its
//! presence, is a function of an upstream fact's payload - so a stale downstream value is
//! observably different from a fresh one. A graph where both computations trivially agree
//! (no payload actually derived from the changed input) would assert nothing; `Test_
//! Skipping_The_Reported_Dependent_Set_Disagrees_With_A_Clean_Rebuild` below exists to prove
//! this fixture is not that.
//!
//! `crates/languages/nomos-lang-rust/src/rollup` is the real shipping producer of a derived
//! hop (`OD-ANALYSIS-002`: the module index depends on its members' syntax facts). It cannot
//! be reached from here - `nomos-lang-rust` depends on `nomos-analysis`, not the reverse, so
//! a dependency the other way would be circular - and the property under test belongs to
//! `MemoryFactStore`'s own invalidate/materialize mechanics, not to any one capability's
//! business logic. A fixture built directly on the store's real API (`Materialize`,
//! `Invalidate`, `Current`), with a genuine content-derived downstream payload, exercises
//! that mechanics exactly as a shipping rollup would.

use nomos_analysis::{
    Dependency, FactKey, FactPayload, FactStore, GenerationCause, GuaranteeDigest, InputDigest,
    MaterializedFact, MemoryFactStore, ReadOutcome,
};
use nomos_contracts::{
    Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128,
    EvidenceClass, FactVariant, GenerationId, Guarantee, IncrementalGranularity, ProviderId,
    SchemaId, SnapshotId, SubjectId,
};

/// The components every fixture key shares: one variant, one configuration and one snapshot
/// throughout, so the two keys differ only in the subject they name.
const FIXTURE_SNAPSHOT_SEED: u8 = 2;
const FIXTURE_VARIANT_SEED: u8 = 3;
const FIXTURE_CONFIGURATION_SEED: u8 = 4;

/// The derived fact's subject: its own, not the leaf's, so the only thing that reaches it is
/// the dependency edge the downstream fact records.
const DOWNSTREAM_SUBJECT_SEED: u8 = 2;

/// Whether the downstream fact is materialized with the dependency edge that makes it
/// followable, or without it.
#[derive(Clone, Copy)]
enum DependencyEdge
{
    /// The shape a real producer has: the downstream fact records what it read, so the walk
    /// reaches it.
    Recorded,
    /// The shape an under-invalidating producer has: no edge is recorded, so nothing follows
    /// the change and the stale entry survives.
    Omitted,
}

/// The two facts this fixture is about: the leaf, and the derived fact that reads it.
struct FixtureFacts
{
    upstream: MaterializedFact,
    downstream: MaterializedFact,
}

fn Digest_From_Seed(seed: u8) -> Digest128
{
    return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
}

fn Subject_Id_From_Seed(seed: u8) -> SubjectId
{
    return SubjectId::From_Digest(Digest_From_Seed(seed));
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

/// The leaf: one subject, fixed address, whatever bytes a test hands it.
fn Upstream_Key() -> FactKey
{
    return FactKey {
        contract: CapabilityId::New("nomos.cap.fixture.leaf"),
        contract_version: ContractVersion::New(1, 0),
        subject: Subject_Id_From_Seed(1),
        semantic_inputs: InputDigest::Of(&[b"fixture"]),
        provider: ProviderId::New("nomos.provider.fixture.leaf"),
        provider_version: ContractVersion::New(1, 0),
        guarantee: GuaranteeDigest::Of(&Fixture_Guarantee()),
        variant: BuildVariantId::From_Digest(Digest_From_Seed(FIXTURE_VARIANT_SEED)),
        configuration: ConfigurationId::From_Digest(Digest_From_Seed(FIXTURE_CONFIGURATION_SEED)),
    };
}

/// The derived hop: a different subject of its own - not the upstream's - per `OD-ANALYSIS-
/// 002`'s load-bearing rule that a rollup keyed on its own input would be named `direct`
/// rather than `dependent` and prove nothing about edge-following.
fn Downstream_Key() -> FactKey
{
    return FactKey {
        contract: CapabilityId::New("nomos.cap.fixture.rollup"),
        contract_version: ContractVersion::New(1, 0),
        subject: Subject_Id_From_Seed(DOWNSTREAM_SUBJECT_SEED),
        semantic_inputs: InputDigest::Of(&[b"fixture-rollup"]),
        provider: ProviderId::New("nomos.provider.fixture.rollup"),
        provider_version: ContractVersion::New(1, 0),
        guarantee: GuaranteeDigest::Of(&Fixture_Guarantee()),
        variant: BuildVariantId::From_Digest(Digest_From_Seed(FIXTURE_VARIANT_SEED)),
        configuration: ConfigurationId::From_Digest(Digest_From_Seed(FIXTURE_CONFIGURATION_SEED)),
    };
}

/// A real recomputation, not a stand-in: the derived fact's payload is the upstream bytes
/// reversed. Changing the upstream bytes changes this output, which is what makes a stale
/// downstream value observably different from a fresh one.
fn Rollup_Of_Upstream_Payload(upstream_payload: &[u8]) -> Vec<u8>
{
    let mut reversed = upstream_payload.to_vec();
    reversed.reverse();

    return reversed;
}

fn Upstream_Fact(payload: &[u8], generation: GenerationId) -> MaterializedFact
{
    return MaterializedFact {
        identity: Upstream_Key().At(generation),
        snapshot: SnapshotId::From_Digest(Digest_From_Seed(FIXTURE_SNAPSHOT_SEED)),
        evidence: EvidenceClass::Verified,
        guarantee: Fixture_Guarantee(),
        payload: FactPayload::New(SchemaId::New("nomos.fixture.leaf.v1"), payload.to_vec()),
    };
}

fn Downstream_Fact(upstream_payload: &[u8], generation: GenerationId) -> MaterializedFact
{
    return MaterializedFact {
        identity: Downstream_Key().At(generation),
        snapshot: SnapshotId::From_Digest(Digest_From_Seed(FIXTURE_SNAPSHOT_SEED)),
        evidence: EvidenceClass::Derived,
        guarantee: Fixture_Guarantee(),
        payload: FactPayload::New(
            SchemaId::New("nomos.fixture.rollup.v1"),
            Rollup_Of_Upstream_Payload(upstream_payload),
        ),
    };
}

fn Downstream_Dependency() -> Dependency
{
    return Dependency {
        key: Upstream_Key(),
        outcome: ReadOutcome::Materialized,
    };
}

/// Materializes the leaf and its one dependent, at `GenerationId::INITIAL`, then changes the
/// leaf's input and recomputes - trusting the store's own `InvalidationReport` in full
/// (`report.direct` and `report.dependent` both) to say what needs it, exactly as a real
/// caller would, rather than assuming the answer.
///
/// `edge` decides whether the downstream fact records a dependency at all.
/// [`DependencyEdge::Omitted`] materializes it with no edge — what an under-invalidating
/// producer looks like from the store's side, since `Follow_Edges` walks exactly the edges
/// `Materialize` was given. With no edge, the real `Invalidate` correctly reports nothing to
/// follow, the loop below never touches the downstream fact, and its original, now-stale
/// entry is what `Current` hands back. It exists to be exercised by the negative-control test
/// below, not by the correctness test.
fn Recompute_Incrementally(
    initial_upstream: &[u8],
    changed_upstream: &[u8],
    edge: DependencyEdge,
) -> FixtureFacts
{
    let mut store = MemoryFactStore::New();
    let dependency_edges: Vec<Dependency> = match edge
    {
        DependencyEdge::Recorded => vec![Downstream_Dependency()],
        DependencyEdge::Omitted => Vec::new(),
    };

    Materialize_Pair(&mut store, initial_upstream, GenerationId::INITIAL, &dependency_edges);

    let gen1 = GenerationId::INITIAL.Next();
    let named = Invalidated_Names(&mut store, gen1);
    Rematerialize_Named(&mut store, &named, changed_upstream, &dependency_edges);

    return Current_After_Recompute(&store, gen1);
}

/// Materializes both fixture facts at `generation`, from `upstream_payload`, with
/// `dependency_edges` recorded against the downstream fact.
fn Materialize_Pair(
    store: &mut MemoryFactStore,
    upstream_payload: &[u8],
    generation: GenerationId,
    dependency_edges: &[Dependency],
)
{
    let upstream_fact = Upstream_Fact(upstream_payload, generation);
    store.Materialize(upstream_fact, &[]).expect("materializes upstream");

    let downstream_fact = Downstream_Fact(upstream_payload, generation);
    store.Materialize(downstream_fact, dependency_edges).expect("materializes downstream");
}

/// Invalidates the upstream subject at `gen1` and returns every fact key the report named,
/// direct and dependent together, in the order a real caller would rematerialize them.
fn Invalidated_Names(store: &mut MemoryFactStore, gen1: GenerationId) -> Vec<FactKey>
{
    let report = store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: Upstream_Key().subject,
            granularity: IncrementalGranularity::File,
        },
        gen1,
    );

    let mut named: Vec<FactKey> = report.direct.clone();
    named.extend(report.dependent.clone());

    return named;
}

/// Rematerializes whichever of `named`'s keys this fixture recognizes, at
/// `GenerationId::INITIAL.Next()`, from `changed_upstream`.
fn Rematerialize_Named(
    store: &mut MemoryFactStore,
    named: &[FactKey],
    changed_upstream: &[u8],
    dependency_edges: &[Dependency],
)
{
    let gen1 = GenerationId::INITIAL.Next();

    for key in named
    {
        if *key == Upstream_Key()
        {
            let fact = Upstream_Fact(changed_upstream, gen1);
            store.Materialize(fact, &[]).expect("rematerializes upstream");
        }
        else if *key == Downstream_Key()
        {
            let fact = Downstream_Fact(changed_upstream, gen1);
            store.Materialize(fact, dependency_edges).expect("rematerializes downstream");
        }
    }
}

/// Reads both fixture facts back at `gen1`, the state the incremental path above ends at.
fn Current_After_Recompute(store: &MemoryFactStore, gen1: GenerationId) -> FixtureFacts
{
    let upstream_now = store
        .Current(&Upstream_Key().At(gen1), gen1)
        .expect("upstream must be readable after the incremental recompute");
    let downstream_now = store
        .Current(&Downstream_Key().At(gen1), gen1)
        .expect(
            "downstream must be readable after the incremental recompute - if this is absent \
             rather than merely stale, the fixture's negative control needs a different shape",
        );

    return FixtureFacts {
        upstream: upstream_now,
        downstream: downstream_now,
    };
}

/// Builds the same two facts from an empty store, at the input state the incremental path
/// ends at, with no history and nothing to invalidate.
fn Recompute_From_Empty(upstream_payload: &[u8], generation: GenerationId) -> FixtureFacts
{
    let mut store = MemoryFactStore::New();
    Materialize_Pair(&mut store, upstream_payload, generation, &[Downstream_Dependency()]);

    let upstream = store
        .Current(&Upstream_Key().At(generation), generation)
        .expect("upstream must be readable in a clean build");
    let downstream = store
        .Current(&Downstream_Key().At(generation), generation)
        .expect("downstream must be readable in a clean build");

    return FixtureFacts { upstream, downstream };
}

/// The property this file exists to state: incremental recomputation after a change agrees
/// with recomputation from empty, over a graph with a real derived hop, comparing fact
/// identity and value together rather than a count.
#[test]
fn Test_Incremental_Recomputation_After_A_Change_Agrees_With_A_Clean_Rebuild()
{
    let initial = b"alpha".to_vec();
    let changed = b"omega-longer".to_vec();
    assert_ne!(
        Rollup_Of_Upstream_Payload(&initial),
        Rollup_Of_Upstream_Payload(&changed),
        "fixture does not exercise an observable stale-vs-fresh difference in the derived fact"
    );

    let gen1 = GenerationId::INITIAL.Next();

    let incremental = Recompute_Incrementally(&initial, &changed, DependencyEdge::Recorded);
    let clean = Recompute_From_Empty(&changed, gen1);

    assert_eq!(
        incremental.upstream, clean.upstream,
        "the leaf fact recomputed incrementally does not agree with a clean recomputation"
    );
    assert_eq!(
        incremental.downstream, clean.downstream,
        "the derived fact recomputed incrementally does not agree with a clean recomputation - \
         this is what an under-invalidating store serves: a stale answer that looks correct"
    );
}

/// Proves the test above is not vacuous. A downstream fact materialized with no dependency
/// edge - what an under-invalidating producer looks like from the store's side - is never
/// named in `report.dependent`, is never rematerialized, and its stale value survives
/// `Current` undetected. If this comparison ever stops disagreeing, the fixture no longer
/// discriminates a correct invalidation from a broken one and the test above would pass for
/// the wrong reason.
#[test]
fn Test_An_Unrecorded_Dependency_Edge_Leaves_A_Stale_Downstream_Value_That_Disagrees_With_A_Clean_Rebuild()
{
    let initial = b"alpha".to_vec();
    let changed = b"omega-longer".to_vec();
    let gen1 = GenerationId::INITIAL.Next();

    let incremental = Recompute_Incrementally(&initial, &changed, DependencyEdge::Omitted);
    let clean = Recompute_From_Empty(&changed, gen1);

    assert_ne!(
        incremental.downstream.payload.bytes, clean.downstream.payload.bytes,
        "skipping the reported dependent set should have left a stale downstream value - if it \
         no longer does, this fixture cannot tell a correct invalidation from a broken one"
    );
}
