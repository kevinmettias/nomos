//! The real crate-to-crate seams `nomos-lang-go` reaches across, driven only through this
//! crate's own public API — the same view a real consumer has.
//!
//! `nomos-lang-go` depends on five other crates in its source: `nomos_analysis`,
//! `nomos_cap_syntax`, `nomos_capability`, `nomos_contracts` and `nomos_model`. Each test
//! below exercises the actual call this crate makes into one of them.

use nomos_lang_go::{Declared_Guarantee, FactContext, Materialization, Materialize_Syntax_Fact, Provider_Offer};
use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};

fn Subject_Id_For_Path(path: &str) -> nomos_contracts::SubjectId
{
    use nomos_model::Content_Digest;

    return nomos_contracts::SubjectId::From_Digest(Content_Digest(path.as_bytes()));
}

/// Fill bytes distinct enough that the three digests below differ from one another; each
/// value carries no meaning beyond "not equal to the others".
const VARIANT_DIGEST_FILL: u8 = 2;
const CONFIGURATION_DIGEST_FILL: u8 = 3;

fn Context() -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}

fn A_Materialized_Fact() -> nomos_analysis::MaterializedFact
{
    return match Materialize_Syntax_Fact(Subject_Id_For_Path("main.go"), "package main\n\nfunc One() {}\n", Context())
    {
        Materialization::Materialized(fact) => *fact,
        Materialization::Unparseable(failure) => panic!("expected a fact: {failure}"),
    };
}

/// `nomos_analysis`: the fact this crate materializes is a real
/// `nomos_analysis::MaterializedFact` that `nomos_analysis`'s own store accepts, and reads
/// back under the key this crate computed — the happy-path flow across the boundary.
#[test]
fn Test_A_Materialized_Fact_Should_Be_Accepted_And_Read_Back_By_Nomos_Analysiss_Own_Store()
{
    use nomos_analysis::{FactStore, MemoryFactStore};

    let fact = A_Materialized_Fact();
    let key = fact.Key().clone();
    let mut store = MemoryFactStore::New();

    store
        .Materialize(fact, &[])
        .expect("nomos_analysis accepts this crate's own fact shape without complaint");

    let current = store
        .Current(&key.At(GenerationId::INITIAL), GenerationId::INITIAL)
        .expect("nomos_analysis can address the fact this crate wrote by the key this crate computed");
    assert_eq!(current.guarantee, Declared_Guarantee());
}

/// `nomos_analysis`: the lifecycle the other side imposes — two different subjects
/// materialized into the same store are two independent entries, addressed by their own
/// keys rather than colliding.
#[test]
fn Test_Two_Different_Subjects_Should_Be_Two_Independent_Entries_In_Nomos_Analysiss_Store()
{
    use nomos_analysis::{FactStore, MemoryFactStore};

    let mut store = MemoryFactStore::New();
    let one = A_Materialized_Fact();
    let one_key = one.Key().clone();
    let two = match Materialize_Syntax_Fact(Subject_Id_For_Path("other.go"), "package main\n\nfunc Two() {}\n", Context())
    {
        Materialization::Materialized(fact) => *fact,
        Materialization::Unparseable(failure) => panic!("expected a fact: {failure}"),
    };
    let two_key = two.Key().clone();

    store.Materialize(one, &[]).expect("the first subject's fact is written");
    store.Materialize(two, &[]).expect("the second subject's fact is written");

    assert_ne!(one_key.Digest(), two_key.Digest(), "different subjects must key apart");
    assert!(store.Current(&one_key.At(GenerationId::INITIAL), GenerationId::INITIAL).is_some());
    assert!(store.Current(&two_key.At(GenerationId::INITIAL), GenerationId::INITIAL).is_some());
}

/// `nomos_cap_syntax`: this crate's own payload bytes decode under the shared schema
/// reader — the property that makes this provider's encoding an interface rather than a
/// private format, moved here from an inline test so it is checked through the public API
/// a real consumer has.
#[test]
fn Test_This_Crates_Payload_Should_Decode_Under_Nomos_Cap_Syntaxs_Own_Reader()
{
    let fact = A_Materialized_Fact();

    let payload = nomos_cap_syntax::Parse_Payload(&fact.payload.bytes)
        .expect("this crate writes the schema nomos_cap_syntax's own reader expects");

    assert_eq!(payload.items.len(), 1);
    assert_eq!(payload.items.first().expect("one item").Own_Name(), "One");
}

/// `nomos_capability`: this crate's own offer is accepted under `nomos_cap_syntax`'s
/// contract through `nomos_capability::Registry` — the composition-time check every real
/// caller relies on, moved here from an inline test so it is checked through the public
/// API a real consumer has.
#[test]
fn Test_This_Crates_Offer_Should_Be_Accepted_By_Nomos_Capabilitys_Own_Registry()
{
    use nomos_capability::Registry;

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_syntax::Capability_Contract())
        .expect("the contract is the first declaration in a fresh registry");

    assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
}

/// `nomos_capability`: the lifecycle the registry imposes — a second declaration of the
/// same capability is refused, not silently replaced.
#[test]
fn Test_A_Second_Declaration_Of_The_Same_Capability_Should_Be_Refused_By_The_Registry()
{
    use nomos_capability::Registry;

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_syntax::Capability_Contract())
        .expect("the first declaration succeeds");

    assert!(
        registry.Declare(nomos_cap_syntax::Capability_Contract()).is_err(),
        "a capability declared twice is a defect the registry must catch, not absorb"
    );
}

/// `nomos_contracts`: the offer's own guarantee is a real `nomos_contracts::Guarantee`
/// that satisfies the contract's ceiling through `Guarantee::Satisfies` — the property
/// `nomos_capability::Registry::Offer` checks internally, asserted directly here against
/// the shared type both sides agree on.
#[test]
fn Test_The_Declared_Guarantee_Should_Satisfy_Nomos_Cap_Syntaxs_Own_Ceiling_Via_Nomos_Contracts()
{
    use nomos_cap_syntax::Ceiling;

    assert!(
        Ceiling().Satisfies(&Declared_Guarantee()),
        "this crate's own claim must satisfy the capability's ceiling, checked through \
         nomos_contracts::Guarantee::Satisfies -- the exact comparison the registry runs"
    );
}

/// `nomos_model`: every subject this crate's public API takes is a `nomos_model`-backed
/// `nomos_contracts::SubjectId` -- the same construction any real caller performs before
/// calling `Materialize_Syntax_Fact`, and two calls over the same path reach the same
/// subject.
#[test]
fn Test_Two_Subjects_Built_Over_The_Same_Path_Should_Be_The_Same_Subject()
{
    assert_eq!(Subject_Id_For_Path("main.go"), Subject_Id_For_Path("main.go"));
    assert_ne!(Subject_Id_For_Path("main.go"), Subject_Id_For_Path("other.go"));
}
