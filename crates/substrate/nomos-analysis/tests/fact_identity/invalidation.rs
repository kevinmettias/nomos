//! What a change discards, what it keeps, and how far along the edges it travels.

use crate::key::{Base, Fact, SEMANTIC, Snapshot, Stored, Subject};
use nomos_analysis::{
    Dependency, FactKey, FactStore, GenerationCause, InvalidationReport, MemoryFactStore,
    ReadOutcome,
};
use nomos_contracts::{CapabilityId, GenerationId, IncrementalGranularity, SubjectId};
use std::collections::BTreeSet;

/// The seed of a subject that is deliberately not the one under test, so a fact about it
/// can only be reached by the behaviour under examination.
const OTHER_SUBJECT_SEED: u8 = 9;

/// The seed of the subject that reads the invalidated one, directly or through a middle
/// fact of the chain.
const CONSUMER_SUBJECT_SEED: u8 = 7;

/// The seed of the subject at the far end of the chain, which reads the consumer rather
/// than the invalidated fact.
const OUTERMOST_SUBJECT_SEED: u8 = 8;

/// The snapshots a replacement moves between: the one the store was materialized at, and
/// the one that replaces it.
const FROM_SNAPSHOT_SEED: u8 = 2;
const TO_SNAPSHOT_SEED: u8 = 9;

/// How many edges the `[read, middle, outer]` chain holds, and the position of `outer` in
/// it: `middle` reads `read`, and `outer` reads `middle`.
const CHAIN_EDGE_COUNT: usize = 2;
const OUTERMOST_POSITION: usize = 2;

/// A fact for `key` that reached for `upstream` and got the given outcome.
fn Materialize_Reading(
    store: &mut MemoryFactStore,
    key: &FactKey,
    upstream: &FactKey,
    outcome: ReadOutcome,
)
{
    let fact = Fact(key, GenerationId::INITIAL);
    let edge = Dependency {
        key: upstream.clone(),
        outcome,
    };

    store.Materialize(fact, &[edge]).expect("materializes");
}

/// What a change to one subject invalidates, at file granularity.
fn Subject_Changed(store: &mut MemoryFactStore, subject: SubjectId, next: GenerationId)
    -> InvalidationReport
{
    return store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject,
            granularity: IncrementalGranularity::File,
        },
        next,
    );
}

/// What a checkout invalidates, naming the members that differ.
fn Snapshot_Replaced(
    store: &mut MemoryFactStore,
    differing: BTreeSet<SubjectId>,
    next: GenerationId,
) -> InvalidationReport
{
    return store.Invalidate(
        &GenerationCause::SnapshotReplaced {
            from: Snapshot(FROM_SNAPSHOT_SEED),
            to: Snapshot(TO_SNAPSHOT_SEED),
            differing,
        },
        next,
    );
}

/// A checkout invalidates what actually differs.
///
/// The cause used to name only the new snapshot and match it against a key component, which
/// meant every fact in the store — the whole corpus, for a checkout that touched one file.
/// It now names the members that differ, which is what the caller doing the replacing
/// already has: two content-addressed maps, and the paths where they disagree.
#[test]
fn Test_Replacing_A_Snapshot_Should_Invalidate_Exactly_The_Members_That_Differ()
{
    let changed = Base();
    let mut untouched = Base();
    untouched.subject = Subject(OTHER_SUBJECT_SEED);
    let mut store = Stored(&changed);
    let fact = Fact(&untouched, GenerationId::INITIAL);
    store.Materialize(fact, &[]).expect("materializes");
    let next = GenerationId::INITIAL.Next();

    let report = Snapshot_Replaced(&mut store, BTreeSet::from([changed.subject]), next);

    assert_eq!(report.direct, vec![changed]);
    assert!(
        store.Current(&untouched.At(next), next).is_some(),
        "a checkout that touched one file discarded a fact about another"
    );
    assert_eq!(
        report.cause.Granularity(),
        IncrementalGranularity::File,
        "a replacement that names its differing members is a statement about files, and \
         reporting it as WholeWorkspace makes every provider's broadening look unavoidable"
    );
}

/// A replacement that changes no member is visible as one.
///
/// Two states can hold identical members and differ in variant or configuration, and those
/// have causes of their own. What must not happen is that a caller whose diff iterated zero
/// times reads a clean result — the prototype's most repeated defect, one level down.
#[test]
fn Test_A_Replacement_That_Differs_In_Nothing_Should_Say_So()
{
    let key = Base();
    let mut store = Stored(&key);
    let next = GenerationId::INITIAL.Next();

    let report = Snapshot_Replaced(&mut store, BTreeSet::new(), next);

    assert_eq!(report.Invalidated(), 0);
    assert!(
        report.Report().contains("0 member(s) differ"),
        "the report reads as a clean invalidation rather than as an empty one: {}",
        report.Report()
    );
    assert!(
        store.Current(&key.At(next), next).is_some(),
        "nothing differed, so nothing may be discarded"
    );
}

#[test]
fn Test_A_Changed_Subject_Should_Invalidate_Its_Facts()
{
    let key = Base();
    let mut store = Stored(&key);
    let next = GenerationId::INITIAL.Next();

    let report = Subject_Changed(&mut store, key.subject, next);

    assert_eq!(report.direct, vec![key.clone()]);
    assert!(store.Current(&key.At(next), next).is_none(), "an invalidated fact read as current");
}

#[test]
fn Test_An_Unrelated_Fact_Should_Be_Retained()
{
    let key = Base();
    let mut elsewhere = Base();
    elsewhere.subject = Subject(OTHER_SUBJECT_SEED);
    let mut store = Stored(&key);
    let fact = Fact(&elsewhere, GenerationId::INITIAL);
    store.Materialize(fact, &[]).expect("materializes");
    let next = GenerationId::INITIAL.Next();

    let report = Subject_Changed(&mut store, key.subject, next);

    assert_eq!(report.Invalidated(), 1, "invalidation flushed the store");
    assert!(
        store.Current(&elsewhere.At(next), next).is_some(),
        "a fact about another subject was invalidated"
    );
    assert_eq!(report.retained, 1);
}

#[test]
fn Test_Invalidation_Should_Follow_Dependency_Edges()
{
    let read = Base();
    let mut derived = Base();
    derived.contract = CapabilityId::New(SEMANTIC);
    derived.subject = Subject(CONSUMER_SUBJECT_SEED);
    let mut store = Stored(&read);
    Materialize_Reading(&mut store, &derived, &read, ReadOutcome::Materialized);
    let next = GenerationId::INITIAL.Next();

    let report = Subject_Changed(&mut store, read.subject, next);

    assert_eq!(report.direct, vec![read]);
    assert_eq!(
        report.dependent,
        vec![derived.clone()],
        "the consumer of an invalidated fact stayed current"
    );
    assert!(store.Current(&derived.At(next), next).is_none());
}

/// Which position in a `[read, middle, outer]` triple each key depends on, forming a chain:
/// `middle` reads `read`, and `outer` reads `middle`.
fn Chain_Edges() -> [(usize, usize); CHAIN_EDGE_COUNT]
{
    return [(1, 0), (OUTERMOST_POSITION, 1)];
}

/// A store holding `read`, its consumer, and the fact that consumes the consumer.
fn Chained_Store(chain: &[&FactKey]) -> MemoryFactStore
{
    let mut store = Stored(*chain.first().expect("every caller hands a chain holding the read key first"));
    for (downstream, upstream) in Chain_Edges()
    {
        Materialize_Reading(
            &mut store,
            *chain.get(downstream).expect("Chain_Edges only names indices within the 3-element chain"),
            *chain.get(upstream).expect("Chain_Edges only names indices within the 3-element chain"),
            ReadOutcome::Materialized,
        );
    }

    return store;
}

#[test]
fn Test_Invalidation_Should_Follow_Edges_Transitively()
{
    let read = Base();
    let mut middle = Base();
    middle.subject = Subject(CONSUMER_SUBJECT_SEED);
    let mut outer = Base();
    outer.subject = Subject(OUTERMOST_SUBJECT_SEED);
    let chain = [&read, &middle, &outer];

    let mut store = Chained_Store(&chain);
    let next = GenerationId::INITIAL.Next();

    let report = Subject_Changed(&mut store, read.subject, next);
    let mut expected = vec![middle, outer];
    expected.sort();

    assert_eq!(report.dependent, expected, "invalidation stopped one edge short");
}

#[test]
fn Test_An_Edge_Recorded_By_A_Missed_Read_Should_Still_Carry_Invalidation()
{
    let absent = Base();
    let mut consumer = Base();
    consumer.subject = Subject(CONSUMER_SUBJECT_SEED);

    let mut store = MemoryFactStore::New();
    let appearing = Fact(&absent, GenerationId::INITIAL);
    Materialize_Reading(&mut store, &consumer, &absent, ReadOutcome::Absent);
    store.Materialize(appearing, &[]).expect("materializes");
    let next = GenerationId::INITIAL.Next();

    let report = Subject_Changed(&mut store, absent.subject, next);

    assert_eq!(
        report.dependent,
        vec![consumer],
        "a judgement made because a fact was missing survived that fact arriving"
    );
}
