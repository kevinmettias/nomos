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
        variant: BuildVariantId::From_Digest(Digest(3)),
        configuration: ConfigurationId::From_Digest(Digest(4)),
    };
}

fn Fact_For(key: &FactKey) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.clone().At(GenerationId::INITIAL),
        snapshot: SnapshotId::From_Digest(Digest(2)),
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

/// A depends on B depends on C depends on D. Invalidating D (the root cause) must let a
/// caller rematerialize D, then C, then B, then A — one singleton group per fact, in that
/// order.
///
/// The assertion reads `Condensation_Of`'s output directly rather than `dependent`'s
/// traversal order: `dependent` is filled by a walk whose entry point and pop order are
/// incidental to this graph's shape, and `Condensation_Of` is computed fresh from the
/// store's own dependency edges among the invalidated keys, so it does not vary with where
/// the walk entered.
#[test]
fn Test_A_Chain_Should_Condense_Into_One_Singleton_Group_Per_Fact_In_Dependency_Order()
{
    let a = Key_For(1);
    let b = Key_For(2);
    let c = Key_For(3);
    let d = Key_For(4);

    let mut store = MemoryFactStore::New();
    Depends_On(&mut store, &d, &[]);
    Depends_On(&mut store, &c, &[&d]);
    Depends_On(&mut store, &b, &[&c]);
    Depends_On(&mut store, &a, &[&b]);

    let report = Subject_Changed(&mut store, d.subject);
    assert_eq!(report.direct, vec![d.clone()]);
    let mut expected_dependent = vec![c.clone(), b.clone(), a.clone()];
    expected_dependent.sort();
    assert_eq!(report.dependent, expected_dependent, "the chain was not fully invalidated");

    let groups = Condensation_Of(&report, &store);

    assert_eq!(
        groups,
        vec![
            RematerializationGroup { members: vec![d] },
            RematerializationGroup { members: vec![c] },
            RematerializationGroup { members: vec![b] },
            RematerializationGroup { members: vec![a] },
        ],
        "D must be rematerializable before C before B before A"
    );
}

/// W depends on X depends on Y depends on Z depends on W: a cycle over four facts. It must
/// condense into exactly one group naming all four, with no claim of a total order among
/// them — a cycle is a shape of the dependency graph, not a fault the store refuses.
#[test]
fn Test_A_Four_Cycle_Should_Condense_Into_One_Group_Naming_All_Four()
{
    let w = Key_For(10);
    let x = Key_For(11);
    let y = Key_For(12);
    let z = Key_For(13);

    let mut store = MemoryFactStore::New();
    Depends_On(&mut store, &w, &[&x]);
    Depends_On(&mut store, &x, &[&y]);
    Depends_On(&mut store, &y, &[&z]);
    Depends_On(&mut store, &z, &[&w]);

    let report = Subject_Changed(&mut store, w.subject);
    assert_eq!(report.direct, vec![w.clone()]);
    assert_eq!(report.Invalidated(), 4, "the cycle did not terminate cleanly");

    let groups = Condensation_Of(&report, &store);

    assert_eq!(groups.len(), 1, "a four-cycle produced more than one group: {groups:?}");
    let Some(only) = groups.first()
    else
    {
        panic!("checked len == 1 above");
    };
    let mut members = only.members.clone();
    members.sort();
    let mut expected = vec![w, x, y, z];
    expected.sort();
    assert_eq!(members, expected);
    assert!(only.Is_Cycle(), "four mutually dependent facts were not reported as a cycle");
}

/// P depends on Q; Q and R depend on each other; R also depends on S. An acyclic chain
/// enters the two-fact cycle from P and leaves it toward S.
///
/// `Condensation_Of` must preserve both: the acyclic ordering around the cycle (S before
/// the cycle, the cycle before P), and the mutual-dependency group inside it (Q and R named
/// together, with no order claimed between them).
#[test]
fn Test_A_Chain_Through_A_Cycle_Should_Preserve_Both_The_Order_And_The_Group()
{
    let p = Key_For(20);
    let q = Key_For(21);
    let r = Key_For(22);
    let s = Key_For(23);

    let mut store = MemoryFactStore::New();
    Depends_On(&mut store, &s, &[]);
    Depends_On(&mut store, &r, &[&q, &s]);
    Depends_On(&mut store, &q, &[&r]);
    Depends_On(&mut store, &p, &[&q]);

    let report = Subject_Changed(&mut store, s.subject);
    assert_eq!(report.direct, vec![s.clone()]);
    assert_eq!(report.Invalidated(), 4, "the chain through the cycle did not fully invalidate");

    let groups = Condensation_Of(&report, &store);

    let [first, second, third] = groups.as_slice()
    else
    {
        panic!("expected exactly 3 groups (s, {{q, r}}, p), got {groups:?}");
    };

    assert_eq!(first.members, vec![s], "S must be rematerializable before the cycle it feeds");
    assert!(!first.Is_Cycle());

    let mut cycle_members = second.members.clone();
    cycle_members.sort();
    let mut expected_cycle = vec![q, r];
    expected_cycle.sort();
    assert_eq!(cycle_members, expected_cycle, "the two-fact cycle was not named as one group");
    assert!(second.Is_Cycle(), "Q and R depend on each other and must be reported as a cycle");

    assert_eq!(third.members, vec![p], "P depends on the cycle and must come after it");
    assert!(!third.Is_Cycle());
}

/// A chain of one hundred thousand facts, each depending on the next. `Condensation_Of`
/// visits nodes in digest order rather than construction order, so which node starts the
/// walk is effectively random relative to chain position — at this depth, the first walk
/// alone is over 98% likely to descend tens of thousands of links before it ever returns,
/// which is exactly the shape that overflowed the native stack before `Tarjan::Visit` became
/// iterative. The assertion is unchanged from the small chain test above: one singleton
/// group per fact, root first.
#[test]
fn Test_A_Chain_One_Hundred_Thousand_Deep_Should_Condense_Without_Overflowing_The_Stack()
{
    const DEPTH: u32 = 100_000;

    fn Key_For_Index(index: u32) -> FactKey
    {
        let mut bytes = [0u8; Digest128::BYTE_LENGTH];
        bytes[..4].copy_from_slice(&index.to_be_bytes());

        return FactKey {
            contract: CapabilityId::New("nomos.cap.syntax.tree"),
            contract_version: ContractVersion::New(1, 0),
            subject: SubjectId::From_Digest(Digest128::From_Bytes(bytes)),
            semantic_inputs: InputDigest::Of(&[b"fixture"]),
            provider: ProviderId::New("nomos.provider.fixture"),
            provider_version: ContractVersion::New(1, 0),
            guarantee: GuaranteeDigest::Of(&Fixture_Guarantee()),
            variant: BuildVariantId::From_Digest(Digest(3)),
            configuration: ConfigurationId::From_Digest(Digest(4)),
        };
    }

    let keys: Vec<FactKey> = (0..DEPTH).map(Key_For_Index).collect();

    let Some(root) = keys.last().cloned()
    else
    {
        panic!("DEPTH is nonzero");
    };
    let Some(first_key) = keys.first().cloned()
    else
    {
        panic!("DEPTH is nonzero");
    };

    let mut store = MemoryFactStore::New();
    Depends_On(&mut store, &root, &[]);
    for window in keys.windows(2).rev()
    {
        let [dependent, dependency] = window
        else
        {
            continue;
        };
        Depends_On(&mut store, dependent, &[dependency]);
    }

    let report = Subject_Changed(&mut store, root.subject);
    assert_eq!(report.Invalidated(), DEPTH as usize, "the deep chain did not fully invalidate");

    let groups = Condensation_Of(&report, &store);

    assert_eq!(groups.len(), DEPTH as usize, "expected one singleton group per fact");
    for (position, group) in groups.iter().enumerate()
    {
        assert!(!group.Is_Cycle(), "a chain link was reported as a cycle at position {position}");
    }

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
    assert_eq!(last_group.members, vec![first_key]);
}
