//! `Condensation_Of`: whether a caller can derive a valid rematerialization order from what
//! one invalidation reached, and can tell a mutual dependency from an acyclic one instead of
//! receiving both as one indistinguishable `Vec<FactKey>`.
//!
//! Three graphs, driven against a real `MemoryFactStore` rather than asserted on paper: a
//! chain, a cycle, and a chain that both enters and leaves a cycle.

use nomos_analysis::{
    Condensation_Of, Dependency, FactKey, FactPayload, FactStore, GenerationCause,
    GuaranteeDigest, InputDigest, InvalidationReport, MaterializedFact, MemoryFactStore,
    ReadOutcome, RematerializationGroup,
};
use nomos_contracts::{
    Assurance, BuildVariantId, CapabilityId, ConfigurationId, ContractVersion, Digest128,
    EvidenceClass, FactVariant, GenerationId, Guarantee, IncrementalGranularity, ProviderId,
    SchemaId, SnapshotId, SubjectId,
};

/// The components every fixture key shares: one variant, one configuration and one snapshot
/// throughout, so a graph is told apart only by the subjects its nodes name.
const FIXTURE_SNAPSHOT_SEED: u8 = 2;
const FIXTURE_VARIANT_SEED: u8 = 3;
const FIXTURE_CONFIGURATION_SEED: u8 = 4;

/// How many facts the four-cycle fixture holds, and so how many facts a change to any one of
/// them reaches.
const CYCLE_LENGTH: usize = 4;

/// How many facts the chain-through-a-cycle fixture holds, and so how many facts a change to
/// the fact the chain leaves from reaches.
const CHAIN_THROUGH_CYCLE_LENGTH: usize = 4;

/// How many members the two-fact cycle the chain-through-a-cycle fixture wraps holds.
const TWO_FACT_CYCLE_LENGTH: usize = 2;

/// A `u32` index is four bytes wide, which is how much of a subject digest
/// [`Deep_Chain_Keys`] writes its index into.
const INDEX_BYTES: usize = 4;

/// A chain is materialized one adjacent (dependent, dependency) pair at a time.
const CHAIN_PAIR: usize = 2;

fn Digest(seed: u8) -> Digest128
{
    return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
}

fn Subject(seed: u8) -> SubjectId
{
    return SubjectId::From_Digest(Digest(seed));
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

/// A fact key naming `subject` and otherwise fixed. Every node in every graph this file
/// builds is one of these, distinguished only by which subject it names.
fn Key_For(subject: u8) -> FactKey
{
    return FactKey {
        contract: CapabilityId::New("nomos.cap.syntax.tree"),
        contract_version: ContractVersion::New(1, 0),
        subject: Subject(subject),
        semantic_inputs: InputDigest::Of(&[b"fixture"]),
        provider: ProviderId::New("nomos.provider.fixture"),
        provider_version: ContractVersion::New(1, 0),
        guarantee: GuaranteeDigest::Of(&Fixture_Guarantee()),
        variant: BuildVariantId::From_Digest(Digest(FIXTURE_VARIANT_SEED)),
        configuration: ConfigurationId::From_Digest(Digest(FIXTURE_CONFIGURATION_SEED)),
    };
}

fn Fact_For(key: &FactKey) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.clone().At(GenerationId::INITIAL),
        snapshot: SnapshotId::From_Digest(Digest(FIXTURE_SNAPSHOT_SEED)),
        evidence: EvidenceClass::Derived,
        guarantee: Fixture_Guarantee(),
        payload: FactPayload::New(SchemaId::New("nomos.syntax.v1"), b"tree".to_vec()),
    };
}

/// Materializes `key`, declaring a read of every key in `upstream` — `key` depends on each
/// of them, the same relationship a real reader would have recorded through
/// `Into_Dependencies`.
fn Depends_On(store: &mut MemoryFactStore, key: &FactKey, upstream: &[&FactKey])
{
    let fact = Fact_For(key);
    let edges: Vec<Dependency> = upstream
        .iter()
        .map(|dependency_key| {
            return Dependency {
                key: (*dependency_key).clone(),
                outcome: ReadOutcome::Materialized,
            };
        })
        .collect();

    store.Materialize(fact, &edges).expect("materializes");
}

/// Invalidates exactly the fact(s) `subject` names, at file granularity, and returns the
/// report — the entry point the whole walk spreads out from.
fn Subject_Changed(store: &mut MemoryFactStore, subject: SubjectId) -> InvalidationReport
{
    return store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject,
            granularity: IncrementalGranularity::File,
        },
        GenerationId::INITIAL.Next(),
    );
}

/// The four-node chain `a` → `b` → `c` → `d` in a fresh store, alongside its four keys.
struct Four_Long_Chain
{
    store: MemoryFactStore,
    a: FactKey,
    b: FactKey,
    c: FactKey,
    d: FactKey,
}

/// A depends on B depends on C depends on D, in a fresh store — the fixture graph
/// `Test_A_Chain_Should_Condense_Into_One_Singleton_Group_Per_Fact_In_Dependency_Order`
/// asserts over.
///
/// The assertion reads `Condensation_Of`'s output directly rather than `dependent`'s
/// traversal order: `dependent` is filled by a walk whose entry point and pop order are
/// incidental to this graph's shape, and `Condensation_Of` is computed fresh from the
/// store's own dependency edges among the invalidated keys, so it does not vary with where
/// the walk entered.
fn Four_Long_Chain() -> Four_Long_Chain
{
    /// The subject seed of each node, in the order the chain reads: `a` reads `b` reads `c`
    /// reads `d`. Only distinctness matters — no assertion compares a seed's value.
    const SEED_A: u8 = 1;
    const SEED_B: u8 = 2;
    const SEED_C: u8 = 3;
    const SEED_D: u8 = 4;

    let a = Key_For(SEED_A);
    let b = Key_For(SEED_B);
    let c = Key_For(SEED_C);
    let d = Key_For(SEED_D);

    let mut store = MemoryFactStore::New();
    Depends_On(&mut store, &d, &[]);
    Depends_On(&mut store, &c, &[&d]);
    Depends_On(&mut store, &b, &[&c]);
    Depends_On(&mut store, &a, &[&b]);

    return Four_Long_Chain { store, a, b, c, d };
}

#[test]
fn Test_A_Chain_Should_Condense_Into_One_Singleton_Group_Per_Fact_In_Dependency_Order()
{
    let mut fixture = Four_Long_Chain();

    let report = Subject_Changed(&mut fixture.store, fixture.d.subject);
    assert_eq!(report.direct, vec![fixture.d.clone()]);
    let mut expected_dependent = vec![fixture.c.clone(), fixture.b.clone(), fixture.a.clone()];
    expected_dependent.sort();
    assert_eq!(report.dependent, expected_dependent, "the chain was not fully invalidated");

    let groups = Condensation_Of(&report, &fixture.store);
    assert_eq!(
        groups,
        vec![
            RematerializationGroup { members: vec![fixture.d] },
            RematerializationGroup { members: vec![fixture.c] },
            RematerializationGroup { members: vec![fixture.b] },
            RematerializationGroup { members: vec![fixture.a] },
        ],
        "D must be rematerializable before C before B before A"
    );
}

/// The four-node cycle `w` → `x` → `y` → `z` → `w` in a fresh store, alongside its four keys.
struct Four_Cycle
{
    store: MemoryFactStore,
    w: FactKey,
    x: FactKey,
    y: FactKey,
    z: FactKey,
}

/// W depends on X depends on Y depends on Z depends on W, in a fresh store — the fixture
/// graph `Test_A_Four_Cycle_Should_Condense_Into_One_Group_Naming_All_Four` asserts over.
///
/// A cycle is a shape of the dependency graph, not a fault the store refuses: it must
/// condense into exactly one group naming all four, with no claim of a total order among
/// them.
fn Four_Cycle() -> Four_Cycle
{
    /// The subject seed of each node, in the order the cycle reads. Only distinctness
    /// matters — no assertion compares a seed's value.
    const SEED_W: u8 = 10;
    const SEED_X: u8 = 11;
    const SEED_Y: u8 = 12;
    const SEED_Z: u8 = 13;

    let w = Key_For(SEED_W);
    let x = Key_For(SEED_X);
    let y = Key_For(SEED_Y);
    let z = Key_For(SEED_Z);

    let mut store = MemoryFactStore::New();
    Depends_On(&mut store, &w, &[&x]);
    Depends_On(&mut store, &x, &[&y]);
    Depends_On(&mut store, &y, &[&z]);
    Depends_On(&mut store, &z, &[&w]);

    return Four_Cycle { store, w, x, y, z };
}

#[test]
fn Test_A_Four_Cycle_Should_Condense_Into_One_Group_Naming_All_Four()
{
    let mut fixture = Four_Cycle();

    let report = Subject_Changed(&mut fixture.store, fixture.w.subject);
    assert_eq!(report.direct, vec![fixture.w.clone()]);
    assert_eq!(report.Invalidated(), CYCLE_LENGTH, "the cycle did not terminate cleanly");

    let groups = Condensation_Of(&report, &fixture.store);
    assert_eq!(groups.len(), 1, "a four-cycle produced more than one group: {groups:?}");
    let Some(only) = groups.first()
    else
    {
        panic!("checked len == 1 above");
    };
    Assert_Names_Exactly(only, &[&fixture.w, &fixture.x, &fixture.y, &fixture.z]);
    assert!(only.Is_Cycle(), "four mutually dependent facts were not reported as a cycle");
}

/// Asserts `group` names exactly `keys` and nothing else, whatever order it holds them in —
/// the members one condensed group must carry.
fn Assert_Names_Exactly(group: &RematerializationGroup, keys: &[&FactKey])
{
    let mut members = group.members.clone();
    members.sort();
    let mut expected: Vec<FactKey> = keys.iter().map(|key| return (*key).clone()).collect();
    expected.sort();

    assert_eq!(members, expected, "the group named a different set of facts");
}

/// The chain-through-a-cycle fixture: `p` reads `q`, `q` and `r` read each other, and `r`
/// also reads `s`.
struct Chain_Through_Cycle
{
    store: MemoryFactStore,
    p: FactKey,
    q: FactKey,
    r: FactKey,
    s: FactKey,
}

/// P depends on Q; Q and R depend on each other; R also depends on S, in a fresh store — the
/// fixture graph `Test_A_Chain_Through_A_Cycle_Should_Preserve_Both_The_Order_And_The_Group`
/// asserts over.
///
/// An acyclic chain enters the two-fact cycle from P and leaves it toward S, and
/// `Condensation_Of` must preserve both: the acyclic ordering around the cycle (S before the
/// cycle, the cycle before P), and the mutual-dependency group inside it (Q and R named
/// together, with no order claimed between them).
fn Chain_Through_Cycle() -> Chain_Through_Cycle
{
    /// The subject seed of each node, in the order the chain reads. Only distinctness
    /// matters — no assertion compares a seed's value.
    const SEED_P: u8 = 20;
    const SEED_Q: u8 = 21;
    const SEED_R: u8 = 22;
    const SEED_S: u8 = 23;

    let p = Key_For(SEED_P);
    let q = Key_For(SEED_Q);
    let r = Key_For(SEED_R);
    let s = Key_For(SEED_S);

    let mut store = MemoryFactStore::New();
    Depends_On(&mut store, &s, &[]);
    Depends_On(&mut store, &r, &[&q, &s]);
    Depends_On(&mut store, &q, &[&r]);
    Depends_On(&mut store, &p, &[&q]);

    return Chain_Through_Cycle { store, p, q, r, s };
}

/// Asserts `groups` is exactly three, in order: `before` alone, `cycle`'s two members
/// together as one cycle, then `after` alone — both the acyclic order around the cycle and
/// the mutual-dependency group inside it, which is the property this file's chain-through-a-
/// cycle test exists to prove `Condensation_Of` preserves.
fn Assert_Chain_Around_Cycle(
    groups: &[RematerializationGroup],
    before: FactKey,
    cycle: [FactKey; TWO_FACT_CYCLE_LENGTH],
    after: FactKey,
)
{
    let [first, second, third] = groups
    else
    {
        panic!("expected exactly 3 groups (entering fact, the 2-cycle, leaving fact), got {groups:?}");
    };

    assert_eq!(first.members, vec![before], "the entering fact must be rematerializable before the cycle it feeds");
    assert!(!first.Is_Cycle());

    Assert_Names_Exactly(&second, &[&cycle[0], &cycle[1]]);
    assert!(second.Is_Cycle(), "the cycle members depend on each other and must be reported as a cycle");

    assert_eq!(third.members, vec![after], "the leaving fact depends on the cycle and must come after it");
    assert!(!third.Is_Cycle());
}

#[test]
fn Test_A_Chain_Through_A_Cycle_Should_Preserve_Both_The_Order_And_The_Group()
{
    let mut fixture = Chain_Through_Cycle();

    let report = Subject_Changed(&mut fixture.store, fixture.s.subject);
    assert_eq!(report.direct, vec![fixture.s.clone()]);
    assert_eq!(
        report.Invalidated(),
        CHAIN_THROUGH_CYCLE_LENGTH,
        "the chain through the cycle did not fully invalidate"
    );

    let groups = Condensation_Of(&report, &fixture.store);
    Assert_Chain_Around_Cycle(
        &groups,
        fixture.s.clone(),
        [fixture.q.clone(), fixture.r.clone()],
        fixture.p.clone(),
    );
}

/// `depth` fact keys, distinguished only by an index folded into the subject's digest —
/// large enough, and cheap enough to build, to make a chain over them exercise
/// `Tarjan::Visit`'s iterative walk rather than a native recursion depth nothing here
/// controls.
fn Deep_Chain_Keys(depth: u32) -> Vec<FactKey>
{
    fn Key_For_Index(index: u32) -> FactKey
    {
        let mut bytes = [0u8; Digest128::BYTE_LENGTH];
        bytes[..INDEX_BYTES].copy_from_slice(&index.to_be_bytes());

        return FactKey {
            contract: CapabilityId::New("nomos.cap.syntax.tree"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Digest128::From_Bytes(bytes)),
            semantic_inputs: InputDigest::Of(&[b"fixture"]),
            provider: ProviderId::New("nomos.provider.fixture"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&Fixture_Guarantee()),
            variant: BuildVariantId::From_Digest(Digest(FIXTURE_VARIANT_SEED)),
            configuration: ConfigurationId::From_Digest(Digest(FIXTURE_CONFIGURATION_SEED)),
        };
    }

    return (0..depth).map(Key_For_Index).collect();
}

/// The two ends of a chain built from a key list: the root it starts from, and the leaf it
/// reaches last.
struct Chain_Ends
{
    root: FactKey,
    leaf: FactKey,
}

/// The root (last) and leaf (first) of a chain built from `keys` — panics if `keys` is
/// empty, which only a zero depth could produce.
fn Chain_Ends(keys: &[FactKey]) -> Chain_Ends
{
    let Some(root) = keys.last().cloned()
    else
    {
        panic!("DEPTH is nonzero");
    };
    let Some(leaf) = keys.first().cloned()
    else
    {
        panic!("DEPTH is nonzero");
    };

    return Chain_Ends { root, leaf };
}

/// Materializes `keys` as one chain, root first, each fact depending on the next so its
/// dependency is already present when it is materialized.
fn Deep_Chain_Store(keys: &[FactKey], root: &FactKey) -> MemoryFactStore
{
    let mut store = MemoryFactStore::New();
    Depends_On(&mut store, root, &[]);
    for window in keys.windows(CHAIN_PAIR).rev()
    {
        let [dependent, dependency] = window
        else
        {
            continue;
        };
        Depends_On(&mut store, dependent, &[dependency]);
    }

    return store;
}

/// Asserts every group in `groups` is a singleton, non-cycle — the shape a chain with no
/// mutual dependency must condense into.
fn Assert_All_Singletons(groups: &[RematerializationGroup])
{
    for (position, group) in groups.iter().enumerate()
    {
        assert!(!group.Is_Cycle(), "a chain link was reported as a cycle at position {position}");
    }
}

/// A chain of `depth` facts held in a fresh store, alongside the root and leaf keys that
/// bound it.
struct Deep_Chain
{
    store: MemoryFactStore,
    root: FactKey,
    leaf: FactKey,
}

/// `depth` fact keys chained root-to-leaf in a fresh store, alongside the root and leaf
/// keys — merges [`Deep_Chain_Keys`], [`Chain_Ends`] and [`Deep_Chain_Store`] into the one
/// fixture the deep-chain test asserts over.
fn Deep_Chain(depth: u32) -> Deep_Chain
{
    let keys = Deep_Chain_Keys(depth);
    let ends = Chain_Ends(&keys);
    let store = Deep_Chain_Store(&keys, &ends.root);

    return Deep_Chain {
        store,
        root: ends.root,
        leaf: ends.leaf,
    };
}

/// Asserts `groups`' first group is `root` alone and its last is `leaf` alone — the root end
/// and the leaf end of the chain [`Deep_Chain`] built.
fn Assert_Chain_Ends(groups: &[RematerializationGroup], root: FactKey, leaf: FactKey)
{
    let Some(first_group) = groups.first()
    else
    {
        panic!("checked len == DEPTH above");
    };
    let Some(last_group) = groups.last()
    else
    {
        panic!("checked len == DEPTH above");
    };
    assert_eq!(first_group.members, vec![root]);
    assert_eq!(last_group.members, vec![leaf]);
}

#[test]
fn Test_A_Chain_One_Hundred_Thousand_Deep_Should_Condense_Without_Overflowing_The_Stack()
{
    const DEPTH: u32 = 100_000;
    let mut fixture = Deep_Chain(DEPTH);

    let report = Subject_Changed(&mut fixture.store, fixture.root.subject);
    assert_eq!(report.Invalidated(), DEPTH as usize, "the deep chain did not fully invalidate");

    let groups = Condensation_Of(&report, &fixture.store);
    assert_eq!(groups.len(), DEPTH as usize, "expected one singleton group per fact");
    Assert_All_Singletons(&groups);

    Assert_Chain_Ends(&groups, fixture.root.clone(), fixture.leaf.clone());
}
