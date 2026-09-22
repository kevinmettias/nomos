//! What the fact store costs: writing into it, reading out of it, and spreading an
//! invalidation through it.
//!
//! A measurement, not an assertion about speed: a timing figure is evidence for a commit
//! message and not a gate, because a shared runner under load would fail a threshold that
//! says nothing about the code. What is asserted on every run is what the timings would be
//! worthless without — that the store still answers correctly at the sizes it is timed at,
//! and that a key's history stays bounded however many times it is rewritten.
//!
//! Run it with
//! `cargo test --release -p nomos-analysis --test store_cost -- --ignored --nocapture`.

use nomos_analysis::{
    Dependency, FactKey, FactPayload, FactStore, GenerationCause, GuaranteeDigest, InputDigest,
    MaterializedFact, MemoryFactStore, ReadOutcome,
};
use nomos_contracts::{
    Assurance, BuildVariantId, CapabilityId, ContractVersion, Digest128, EvidenceClass,
    FactVariant, GenerationId, Guarantee, IncrementalGranularity, ProviderId, SchemaId, SnapshotId,
    SubjectId,
};

const V1: ContractVersion = ContractVersion::New(1, 0);

/// How many distinct facts the store holds while it is being read and written.
const FACTS: u32 = 20_000;

/// How many reads the read cost is averaged over. One per fact, so the measurement walks
/// the whole store rather than re-reading one entry the machine has already cached.
const READS: u32 = 20_000;

/// How many bytes a fact's payload carries. A fact is not a scalar — a parse result or a
/// finding set is kilobytes — and a read that copies the payload only costs what the
/// payload is worth, so a measurement taken with an empty one would report nothing.
const PAYLOAD_BYTES: usize = 2_048;

/// How many times the retention assertion rewrites one key, each at a later generation.
const REWRITES: u64 = 64;

/// The generation every fact in the timed store is written at.
const WRITTEN_AT: u64 = 1;

/// The generation the invalidation is applied at, after every write above.
const INVALIDATED_AT: u64 = 2;

/// The variant component of every key here.
const VARIANT_SEED: u8 = 3;

/// The configuration component of every key here, distinct from [`VARIANT_SEED`] so a key
/// assembled with the wrong one is not silently the same key.
const CONFIGURATION_SEED: u8 = 4;

/// The workspace state every fact here was measured against.
const SNAPSHOT_SEED: u8 = 2;

fn Guarantee_Offered() -> Guarantee
{
    return Guarantee::New(
        FactVariant::Syntactic,
        Assurance::Sound,
        Assurance::Unknown,
        IncrementalGranularity::File,
    );
}

/// A subject digest that differs in every byte position that `subject` does, so twenty
/// thousand keys do not share a prefix and a map keyed on them is not measured on its best
/// case.
fn Subject_Digest(subject: u32) -> Digest128
{
    let mut bytes = [0_u8; Digest128::BYTE_LENGTH];
    let raw = subject.to_le_bytes();
    for (slot, byte) in bytes.iter_mut().zip(raw.iter().cycle())
    {
        *slot = *byte;
    }

    return Digest128::From_Bytes(bytes);
}

fn Seeded_Digest(seed: u8) -> Digest128
{
    return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
}

fn Key_Of(subject: u32) -> FactKey
{
    return FactKey {
        contract: CapabilityId::New("nomos.cap.test.store_cost"),
        contract_version: V1,
        subject: SubjectId::From_Digest(Subject_Digest(subject)),
        semantic_inputs: InputDigest::Of(&[b"fn main() {}"]),
        provider: ProviderId::New("nomos.provider.test.store_cost"),
        provider_version: V1,
        guarantee: GuaranteeDigest::Of(&Guarantee_Offered()),
        variant: BuildVariantId::From_Digest(Seeded_Digest(VARIANT_SEED)),
        configuration: ConfigurationId_Seed(),
    };
}

fn ConfigurationId_Seed() -> nomos_contracts::ConfigurationId
{
    return nomos_contracts::ConfigurationId::From_Digest(Seeded_Digest(CONFIGURATION_SEED));
}

fn Fact_Of(key: &FactKey, generation: GenerationId) -> MaterializedFact
{
    return MaterializedFact {
        identity: key.clone().At(generation),
        snapshot: SnapshotId::From_Digest(Seeded_Digest(SNAPSHOT_SEED)),
        evidence: EvidenceClass::Derived,
        guarantee: Guarantee_Offered(),
        payload: FactPayload::New(
            SchemaId::New("nomos.test.store_cost.v1"),
            vec![b'x'; PAYLOAD_BYTES],
        ),
    };
}

/// A store holding [`FACTS`] facts, each depending on the one before it, and the
/// nanoseconds per write it took to build.
fn Built_Store() -> (MemoryFactStore, u128)
{
    let generation = GenerationId::From_Raw(WRITTEN_AT);
    let keys: Vec<FactKey> = (0..FACTS).map(Key_Of).collect();

    let started = std::time::Instant::now();
    let mut store = MemoryFactStore::New();
    for (position, key) in keys.iter().enumerate()
    {
        let upstream = position.checked_sub(1).and_then(|before| return keys.get(before));
        let dependencies: Vec<Dependency> = upstream
            .map(|before| {
                return vec![Dependency { key: before.clone(), outcome: ReadOutcome::Materialized }];
            })
            .unwrap_or_default();
        store
            .Materialize(Fact_Of(key, generation), &dependencies)
            .expect("every key is written once, at one generation");
    }
    let elapsed = started.elapsed().as_nanos();

    return (store, Divided_By(elapsed, FACTS));
}

/// `elapsed` spread over `operations`.
///
/// Written as a checked division with the absent case named, because this workspace denies
/// arithmetic that can panic even in a test: an operation count of zero would time nothing,
/// and reporting that as zero nanoseconds per operation would be a number rather than an
/// answer.
fn Divided_By(elapsed: u128, operations: u32) -> u128
{
    return elapsed
        .checked_div(u128::from(operations))
        .expect("every population timed here is a non-zero constant");
}

/// The nanoseconds one [`FactStore::Current`] read costs, averaged over [`READS`] of them.
fn Nanoseconds_Per_Current_Read(store: &MemoryFactStore) -> u128
{
    let at = GenerationId::From_Raw(WRITTEN_AT);
    let identities: Vec<_> = (0..READS).map(|subject| return Key_Of(subject % FACTS).At(at)).collect();

    let started = std::time::Instant::now();
    let mut bytes: usize = 0;
    for identity in &identities
    {
        bytes = bytes.saturating_add(
            store.Current(identity, at).map_or(0, |fact| return fact.payload.bytes.len()),
        );
    }
    let elapsed = started.elapsed().as_nanos();

    assert_eq!(
        bytes,
        PAYLOAD_BYTES.saturating_mul(READS as usize),
        "a read the timing loop counted on came back empty"
    );

    return Divided_By(elapsed, READS);
}

/// The nanoseconds one [`FactStore::Current_Borrowed`] read costs, over the same keys in the
/// same order as [`Nanoseconds_Per_Current_Read`] — the difference between the two is what
/// copying a fact costs and nothing else.
fn Nanoseconds_Per_Borrowed_Read(store: &MemoryFactStore) -> u128
{
    let at = GenerationId::From_Raw(WRITTEN_AT);
    let identities: Vec<_> = (0..READS).map(|subject| return Key_Of(subject % FACTS).At(at)).collect();

    let started = std::time::Instant::now();
    let mut bytes: usize = 0;
    for identity in &identities
    {
        bytes = bytes.saturating_add(
            store.Current_Borrowed(identity, at).map_or(0, |fact| return fact.payload.bytes.len()),
        );
    }
    let elapsed = started.elapsed().as_nanos();

    assert_eq!(
        bytes,
        PAYLOAD_BYTES.saturating_mul(READS as usize),
        "a read the timing loop counted on came back empty"
    );

    return Divided_By(elapsed, READS);
}

/// The nanoseconds one [`FactKey::Digest`] costs. Not a store operation at all, and reported
/// because it is what a store read used to pay before a key was ever looked up: the digest is
/// hashed from nine separately allocated key components rather than read from a field.
fn Nanoseconds_Per_Key_Digest() -> u128
{
    let keys: Vec<FactKey> = (0..READS).map(|subject| return Key_Of(subject % FACTS)).collect();

    let started = std::time::Instant::now();
    let mut hashed = 0_u32;
    for key in &keys
    {
        if !key.Digest().Bytes().is_empty()
        {
            hashed = hashed.saturating_add(1);
        }
    }
    let elapsed = started.elapsed().as_nanos();

    assert_eq!(hashed, READS, "a digest the timing loop counted on was never computed");

    return Divided_By(elapsed, READS);
}

/// The nanoseconds one whole-store invalidation costs, spreading from the first fact
/// through the chain of dependents behind it.
fn Nanoseconds_Per_Invalidation(store: &mut MemoryFactStore) -> u128
{
    let cause = GenerationCause::SubjectChanged {
        subject: SubjectId::From_Digest(Subject_Digest(0)),
        granularity: IncrementalGranularity::File,
    };

    let started = std::time::Instant::now();
    let report = store.Invalidate(&cause, GenerationId::From_Raw(INVALIDATED_AT));
    let elapsed = started.elapsed().as_nanos();

    assert_eq!(
        report.Invalidated(),
        FACTS as usize,
        "the chain reaches every fact, so an invalidation at its head invalidates all of them"
    );

    return elapsed;
}

#[test]
#[ignore = "a timing measurement, reported into a commit message rather than gated on"]
fn Test_Store_Cost_Should_Be_Reported_For_Writing_Reading_And_Invalidating()
{
    let (mut store, per_write) = Built_Store();
    let per_read = Nanoseconds_Per_Current_Read(&store);
    let per_borrowed = Nanoseconds_Per_Borrowed_Read(&store);
    let per_digest = Nanoseconds_Per_Key_Digest();
    let invalidation = Nanoseconds_Per_Invalidation(&mut store);

    println!("store of {FACTS} facts, {PAYLOAD_BYTES}-byte payloads");
    println!("  materialize:      {per_write} ns/write");
    println!("  Current:          {per_read} ns/read");
    println!("  Current_Borrowed: {per_borrowed} ns/read");
    println!("  FactKey::Digest:  {per_digest} ns/call");
    println!("  Invalidate:       {invalidation} ns for the whole chain");
}

/// The store answers the same thing at this size that it answers at three facts.
#[test]
fn Test_A_Large_Store_Should_Still_Answer_Every_Key_It_Was_Given()
{
    let at = GenerationId::From_Raw(WRITTEN_AT);
    let (store, _) = Built_Store();

    assert_eq!(store.Live(), FACTS as usize);
    for subject in [0, FACTS / 2, FACTS.saturating_sub(1)]
    {
        let identity = Key_Of(subject).At(at);
        let fact = store.Current(&identity, at).expect("every subject written was written once");
        assert_eq!(fact.payload.bytes.len(), PAYLOAD_BYTES);
    }
}

/// The retention rule, read from outside the crate: rewriting one key many times leaves a
/// store that answers exactly as one written once, so nothing behind the newest entry is
/// reachable and a long-lived process does not accumulate what it cannot reach.
#[test]
fn Test_Rewriting_One_Key_Should_Leave_A_Store_That_Answers_As_A_Single_Write_Does()
{
    let key = Key_Of(0);
    let last = GenerationId::From_Raw(REWRITES);

    let mut rewritten = MemoryFactStore::New();
    for generation in 1..=REWRITES
    {
        rewritten
            .Materialize(Fact_Of(&key, GenerationId::From_Raw(generation)), &[])
            .expect("each write is at a later generation than the one before it");
    }
    let mut once = MemoryFactStore::New();
    once.Materialize(Fact_Of(&key, last), &[]).expect("a fresh store holds nothing to conflict with");

    let rewrites = u32::try_from(REWRITES).expect("the rewrite count is a small constant");
    assert_eq!(rewritten.Materializations(), rewrites);
    assert_eq!(once.Materializations(), 1);
    assert_eq!(rewritten.Live(), once.Live());
    assert_eq!(rewritten.Current(&key.clone().At(last), last), once.Current(&key.clone().At(last), last));
    assert_eq!(rewritten.Historical(&key), once.Historical(&key));
    assert_eq!(rewritten.Dependencies_Of(&key), once.Dependencies_Of(&key));
}
