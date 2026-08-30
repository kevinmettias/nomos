//! The real crate-to-crate seams `nomos-lang-rust-scan` reaches across, driven only through
//! this crate's own public API — the same view a real consumer has.
//!
//! `nomos-lang-rust-scan` depends on five other crates in its source: `nomos_analysis`,
//! `nomos_cap_syntax`, `nomos_capability`, `nomos_contracts` and `nomos_model`. Each test
//! below exercises the actual call this crate makes into one of them. Three of the five —
//! `nomos_cap_syntax`, `nomos_contracts` and `nomos_model` — already had an inline
//! `#[cfg(test)]` check inside `src/`; this file adds the thin, public-API-only equivalent
//! beside the other two, which had none.

use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};
use nomos_lang_rust_scan::{Declared_Guarantee, FactContext, Materialize_Syntax_Fact, Provider_Offer};

fn Subject(path: &str) -> nomos_contracts::SubjectId
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
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([
            CONFIGURATION_DIGEST_FILL;
            Digest128::BYTE_LENGTH
        ])),
        generation: GenerationId::INITIAL,
    };
}

/// `nomos_analysis`: the fact this crate materializes is a real
/// `nomos_analysis::MaterializedFact` that `nomos_analysis`'s own store accepts, and reads
/// back under the key this crate computed — the happy-path flow across the boundary. No
/// existing test drove this pairing, so this one is new.
#[test]
fn Test_A_Materialized_Fact_Should_Be_Accepted_And_Read_Back_By_Nomos_Analysiss_Own_Store()
{
    use nomos_analysis::{FactStore, MemoryFactStore};

    let fact = Materialize_Syntax_Fact(Subject("a.rs"), "pub fn one() {}\n", Context());
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
    let one = Materialize_Syntax_Fact(Subject("a.rs"), "pub fn one() {}\n", Context());
    let one_key = one.Key().clone();
    let two = Materialize_Syntax_Fact(Subject("b.rs"), "pub fn two() {}\n", Context());
    let two_key = two.Key().clone();

    store.Materialize(one, &[]).expect("the first subject's fact is written");
    store.Materialize(two, &[]).expect("the second subject's fact is written");

    assert_ne!(one_key.Digest(), two_key.Digest(), "different subjects must key apart");
    assert!(store.Current(&one_key.At(GenerationId::INITIAL), GenerationId::INITIAL).is_some());
    assert!(store.Current(&two_key.At(GenerationId::INITIAL), GenerationId::INITIAL).is_some());
}

/// `nomos_cap_syntax`: this crate's own payload bytes decode under the shared schema
/// reader — the property that makes this provider's encoding an interface rather than a
/// private format. A thin equivalent of `fact_context.rs`'s
/// `Test_This_Providers_Payload_Should_Decode_Under_The_Schemas_Own_Reader`, checked here
/// through the public API a real consumer has rather than through this crate's own private
/// access to its test module.
#[test]
fn Test_This_Crates_Payload_Should_Decode_Under_Nomos_Cap_Syntaxs_Own_Reader()
{
    let fact = Materialize_Syntax_Fact(Subject("a.rs"), "pub fn one() {}\nfn two() {}\n", Context());

    let payload = nomos_cap_syntax::Parse_Payload(&fact.payload.bytes)
        .expect("this crate writes the schema nomos_cap_syntax's own reader expects");

    assert_eq!(payload.unexpanded, 0);
    assert_eq!(payload.items.len(), 2);
    assert!(payload.items.first().expect("two items").Is_Public());
}

/// `nomos_capability`: this crate's own offer is accepted under `nomos_cap_syntax`'s
/// contract through `nomos_capability::Registry` — the composition-time check every real
/// caller relies on. No existing test drove this pairing (this crate's own `guarantee.rs`
/// only compares `Guarantee` values directly, never through a `Registry`), so this one is
/// new.
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
/// same capability is refused, not silently replaced, and refused for the specific reason
/// the registry names rather than for an unexamined error.
#[test]
fn Test_A_Second_Declaration_Of_The_Same_Capability_Should_Be_Refused_By_The_Registry()
{
    use nomos_capability::{Registry, RegistryError, RegistryErrorKind};

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_syntax::Capability_Contract())
        .expect("the first declaration succeeds");

    assert_eq!(
        registry.Declare(nomos_cap_syntax::Capability_Contract()),
        Err(RegistryError {
            capability: nomos_cap_syntax::Capability(),
            kind: RegistryErrorKind::AlreadyDeclared,
        }),
        "a capability declared twice is a defect the registry must catch, not absorb"
    );
}

/// `nomos_contracts`: this crate's own claim is a real `nomos_contracts::Guarantee` that
/// satisfies `nomos_cap_syntax`'s ceiling through `Guarantee::Satisfies` — the exact
/// comparison `nomos_capability::Registry::Offer` runs internally, asserted here directly
/// against the shared type both sides agree on. A thin equivalent of `guarantee.rs`'s own
/// `Guarantee`-comparison tests, moved to reach only public API.
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
/// `nomos_contracts::SubjectId` — the same construction any real caller performs before
/// calling `Materialize_Syntax_Fact`, and two calls over the same path reach the same
/// subject while two different paths do not. A thin equivalent of `fact_context.rs`'s own
/// `Subject_Of_Path` helper, exercised here as a property rather than only used as a
/// fixture.
#[test]
fn Test_Two_Subjects_Built_Over_The_Same_Path_Should_Be_The_Same_Subject()
{
    assert_eq!(Subject("a.rs"), Subject("a.rs"));
    assert_ne!(Subject("a.rs"), Subject("b.rs"));
}
