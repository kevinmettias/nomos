//! What `Require` answers, and the four different ways it declines to.

use crate::key::{
    Base, Context_At, Memory_Store_For_Key, Registry_For_Guarantee, Requirement_For_Guarantee,
    SYNTAX, Subject_Id_From_Seed, Syntactic,
};
use nomos_analysis::{FactReader, InputDigest, MemoryFactStore, ReadOutcome, Reader};
use nomos_contracts::{
    Applicability, Assurance, CapabilityId, FactVariant, GenerationId, Guarantee,
    IncrementalGranularity,
};

#[test]
fn Test_Require_Should_Answer_When_The_Registry_Resolves_And_The_Fact_Exists()
{
    let key = Base();
    let store = Memory_Store_For_Key(&key);
    let registry = Registry_For_Guarantee(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    let fact = reader.Require(
        &CapabilityId::New(SYNTAX),
        &Subject_Id_From_Seed(1),
        InputDigest::Of(&[b"fn main() {}"]),
        &Requirement_For_Guarantee(Syntactic()),
    );

    assert!(fact.is_ok(), "the honest path does not answer, so the refusals prove nothing");
}

#[test]
fn Test_Require_Should_Return_MissingCapability_When_Nothing_Declares_It()
{
    use nomos_capability::Registry;

    let store = Memory_Store_For_Key(&Base());
    let registry = Registry::New();
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    let refusal = reader
        .Require(
            &CapabilityId::New(SYNTAX),
            &Subject_Id_From_Seed(1),
            InputDigest::Of(&[b"fn main() {}"]),
            &Requirement_For_Guarantee(Syntactic()),
        )
        .expect_err("must refuse");

    assert_eq!(refusal, Applicability::MissingCapability);
    assert_ne!(
        refusal,
        Applicability::NotApplicable,
        "coverage debt was reported as a judgement about the subject"
    );
}

#[test]
fn Test_Require_Should_Refuse_A_Provider_Below_The_Requirement()
{
    let store = Memory_Store_For_Key(&Base());
    let registry = Registry_For_Guarantee(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    let semantic = Guarantee::New(
        FactVariant::SemanticallyResolved,
        Assurance::Sound,
        Assurance::Sound,
        IncrementalGranularity::File,
    );
    let refusal = reader
        .Require(
            &CapabilityId::New(SYNTAX),
            &Subject_Id_From_Seed(1),
            InputDigest::Of(&[b"fn main() {}"]),
            &Requirement_For_Guarantee(semantic),
        )
        .expect_err("must refuse");
    let answered = reader.Dependencies().iter().any(|edge| return edge.outcome.Is_Answered());

    assert_eq!(refusal, Applicability::MissingCapability);
    assert!(!answered, "a fact was returned for a requirement no provider reaches");
}

#[test]
fn Test_Require_Should_Not_Fabricate_A_Fact_The_Store_Does_Not_Hold()
{
    let store = MemoryFactStore::New();
    let registry = Registry_For_Guarantee(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    let refusal = reader
        .Require(
            &CapabilityId::New(SYNTAX),
            &Subject_Id_From_Seed(1),
            InputDigest::Of(&[b"fn main() {}"]),
            &Requirement_For_Guarantee(Syntactic()),
        )
        .expect_err("must refuse");

    assert_eq!(refusal, Applicability::DependencyUnavailable);
    assert_eq!(
        reader.Dependencies().first().map(|dependency| dependency.outcome),
        Some(ReadOutcome::Degraded(Applicability::DependencyUnavailable))
    );
}

#[test]
fn Test_Require_Should_Refuse_A_Fact_Whose_Inputs_Differ()
{
    let store = Memory_Store_For_Key(&Base());
    let registry = Registry_For_Guarantee(Syntactic());
    let mut reader = Reader::On(&store, &registry, Context_At(GenerationId::INITIAL));

    let refusal = reader
        .Require(
            &CapabilityId::New(SYNTAX),
            &Subject_Id_From_Seed(1),
            InputDigest::Of(&[b"fn main() { edited }"]),
            &Requirement_For_Guarantee(Syntactic()),
        )
        .expect_err("must refuse");

    assert_eq!(
        refusal,
        Applicability::DependencyUnavailable,
        "a fact about other inputs was served as this subject's analysis"
    );
}
