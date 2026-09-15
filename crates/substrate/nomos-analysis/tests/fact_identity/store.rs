//! What the store will answer, what it refuses to overwrite, and what it keeps as history.

use crate::key::{Base, Fact, Stored};
use nomos_analysis::{FactStore, GenerationCause, MemoryFactStore};
use nomos_contracts::GenerationId;

#[test]
fn Test_A_Fact_Should_Answer_Its_Own_Key()
{
    let key = Base();
    let store = Stored(&key);

    assert!(
        store
            .Current(&key.clone().At(GenerationId::INITIAL), GenerationId::INITIAL)
            .is_some(),
        "the store answers nothing at all, so a miss proves nothing"
    );
}

#[test]
fn Test_A_Later_Generation_Should_Still_Answer_The_Same_Key()
{
    let key = Base();
    let store = Stored(&key);

    assert!(
        store
            .Current(&key.At(GenerationId::INITIAL.Next()), GenerationId::INITIAL.Next())
            .is_some(),
        "a fact stopped answering because the workspace moved on, so nothing is ever reused"
    );
}

#[test]
fn Test_The_Same_Question_Twice_Should_Materialize_Once()
{
    let key = Base();
    let mut store = Stored(&key);
    let materializations = store.Materializations();

    let already = store.Current(&key.clone().At(GenerationId::INITIAL), GenerationId::INITIAL);
    if already.is_none()
    {
        let fact = Fact(&key, GenerationId::INITIAL);
        store.Materialize(fact, &[]).expect("materializes");
    }

    assert_eq!(
        store.Materializations(),
        materializations,
        "the second run recomputed a fact the store already held"
    );
}

#[test]
fn Test_A_Backdated_Materialization_Should_Be_Refused()
{
    let key = Base();
    let mut store = MemoryFactStore::New();
    let ahead = Fact(&key, GenerationId::INITIAL.Next());
    let behind = Fact(&key, GenerationId::INITIAL);
    store.Materialize(ahead, &[]).expect("materializes");

    let refusal = store.Materialize(behind, &[]).expect_err("must refuse");

    assert!(format!("{refusal}").contains("behind"), "{refusal}");
}

#[test]
fn Test_An_Invalidated_Fact_Should_Stay_Readable_As_History()
{
    let key = Base();
    let mut store = Stored(&key);
    let next = GenerationId::INITIAL.Next();
    store.Invalidate(
        &GenerationCause::ConfigurationChanged {
            configuration: key.configuration,
        },
        next,
    );

    let (fact, supersession) = store
        .Historical(&key)
        .expect("the invalidation above superseded the fact Stored materialized");

    assert_eq!(fact.Generation(), GenerationId::INITIAL);
    assert_eq!(supersession.invalidated_at, next);
    assert!(supersession.cause.contains("configuration"), "{}", supersession.cause);
}

#[test]
fn Test_A_Live_Fact_Should_Have_No_History()
{
    assert!(
        Stored(&Base()).Historical(&Base()).is_none(),
        "a current fact was reported as superseded"
    );
}

#[test]
fn Test_The_Store_Trait_Should_Be_Sealed()
{
    let key = Base();
    let store = Stored(&key);
    let sealed: &dyn FactStore = &store;

    assert!(
        sealed
            .Current(&key.At(GenerationId::INITIAL), GenerationId::INITIAL)
            .is_some(),
        "the trait is reachable as an object and answers nothing"
    );
}
