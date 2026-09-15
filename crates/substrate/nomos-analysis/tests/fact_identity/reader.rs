//! Every read leaves an edge behind, including the reads that found nothing.

use crate::key::{
    Base, Context_At, Needing, Offering, SYNTAX, Stored, Subject, Syntactic, Varied,
};
use nomos_analysis::{
    Component, FactReader, FactStore, GenerationCause, InputDigest, MemoryFactStore, ReadOutcome,
    Reader,
};
use nomos_contracts::{CapabilityId, GenerationId, IncrementalGranularity};

/// The reads this file's first test performs: a hit, a miss, and a requirement. Each one
/// leaves exactly one edge on the reader's trail.
const RECORDED_READS: usize = 3;

/// The subject the derived fact is about, which differs from the fact it reads.
const DERIVED_SUBJECT_SEED: u8 = 7;

#[test]
fn Test_Every_Read_Should_Record_An_Edge()
{
    let key = Base();
    let store = Stored(&key);
    let registry = Offering(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    assert!(reader.Get(&key.clone().At(GenerationId::INITIAL)).is_ok());
    assert!(reader.Get(&Varied(Component::Subject).At(GenerationId::INITIAL)).is_err());
    assert!(
        reader
            .Require(
                &CapabilityId::New(SYNTAX),
                &Subject(1),
                InputDigest::Of(&[b"fn main() {}"]),
                &Needing(Syntactic()),
            )
            .is_ok(),
        "the registry offers SYNTAX at this subject's guarantee, so the requirement resolves"
    );

    assert_eq!(reader.Dependencies().len(), RECORDED_READS, "a read left no edge behind");
}

#[test]
fn Test_A_Read_That_Misses_Should_Record_Its_Miss()
{
    let store = MemoryFactStore::New();
    let registry = Offering(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    assert!(reader.Get(&Base().At(GenerationId::INITIAL)).is_err());

    assert_eq!(
        reader.Dependencies().first().map(|dependency| dependency.outcome),
        Some(ReadOutcome::Absent),
        "a miss recorded nothing, so a judgement made without a fact cannot be revisited \
         when the fact arrives"
    );
}

#[test]
fn Test_A_Superseded_Read_Should_Record_Its_Supersession()
{
    let key = Base();
    let mut store = Stored(&key);
    let next = GenerationId::INITIAL.Next();
    store.Invalidate(
        &GenerationCause::SubjectChanged {
            subject: key.subject,
            granularity: IncrementalGranularity::File,
        },
        next,
    );
    let registry = Offering(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(next));

    let refusal = reader.Get(&key.At(next)).expect_err("must refuse");

    assert!(format!("{refusal}").contains("invalidated"), "{refusal}");
    assert_eq!(
        reader.Dependencies().first().map(|dependency| dependency.outcome),
        Some(ReadOutcome::Superseded)
    );
}

#[test]
fn Test_The_Recorded_Edges_Should_Become_The_Stored_Dependencies()
{
    let read = Base();
    let mut derived = Base();
    derived.subject = Subject(DERIVED_SUBJECT_SEED);
    let mut store = Stored(&read);

    let dependencies = {
        let registry = Offering(Syntactic());
        let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));
        assert!(
            reader.Get(&read.clone().At(GenerationId::INITIAL)).is_ok(),
            "the store holds this fact at this context, so the read answers it"
        );
        reader.Into_Dependencies()
    };
    let fact = crate::key::Fact(&derived, GenerationId::INITIAL);
    store.Materialize(fact, &dependencies).expect("materializes");

    assert_eq!(store.Dependencies_Of(&derived).len(), 1);
    assert_eq!(
        store.Dependencies_Of(&derived).first().map(|dependency| dependency.key.clone()),
        Some(read)
    );
}
