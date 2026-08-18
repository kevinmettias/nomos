//! What a fact is addressed by: every component reaches the digest, and nothing else does.

use crate::key::{Base, Configuration, Snapshot, Stored, Subject, Varied};
use nomos_analysis::{Component, FactKey, FactStore, MaterializedFact, MemoryFactStore};
use nomos_contracts::{GenerationId, SnapshotId};

/// The same fact, read from a named tree.
fn Fact_From(key: &FactKey, snapshot: SnapshotId) -> MaterializedFact
{
    let mut fact = crate::key::Fact(key, GenerationId::INITIAL);
    fact.snapshot = snapshot;

    return fact;
}

/// `Component::All()`'s own mirror, named in the doc comment above it.
///
/// The match has no wildcard arm. A variant added to `Component` without a matching arm
/// added here fails this file to *compile*, not merely to pass — the property D-134 asks a
/// closed enum's mirror to have.
#[test]
fn Test_Every_Component_Should_Be_Matched_Exhaustively()
{
    fn Ordinal(component: Component) -> usize
    {
        return match component
        {
            Component::Contract => 0,
            Component::ContractVersion => 1,
            Component::Subject => 2,
            Component::SemanticInputs => 3,
            Component::Provider => 4,
            Component::ProviderVersion => 5,
            Component::Guarantee => 6,
            Component::Variant => 7,
            Component::Configuration => 8,
        };
    }

    for (index, component) in Component::All().iter().enumerate()
    {
        assert_eq!(
            Ordinal(*component),
            index,
            "{} is not matched at the position Component::All() puts it, so the exhaustive \
             match and the universe have drifted apart",
            component.Label()
        );
    }
}

#[test]
fn Test_The_Component_List_Should_Cover_Every_Part_Of_The_Key()
{
    assert_eq!(
        Base().Parts().len(),
        Component::All().len(),
        "a component of the key is not enumerated, so nothing varies it and nothing tests it"
    );
}

#[test]
fn Test_Every_Component_Should_Reach_The_Key_Digest()
{
    let base = Base().Digest();

    for component in Component::All()
    {
        assert_ne!(
            Varied(*component).Digest(),
            base,
            "{} does not reach the digest, so a fact would be reused under a key that does \
             not describe it",
            component.Label()
        );
    }
}

#[test]
fn Test_A_Change_In_Any_One_Component_Should_Miss_The_Cache()
{
    let store = Stored(&Base());

    for component in Component::All()
    {
        let varied = Varied(*component);

        assert!(
            store
                .Current(&varied.At(GenerationId::INITIAL), GenerationId::INITIAL)
                .is_none(),
            "a fact answered a question that differs in {}, which is a cached answer to a \
             different question",
            component.Label()
        );
    }
}

#[test]
fn Test_Two_Components_Should_Not_Cancel_Each_Other_Out()
{
    let mut both = Base();
    both.subject = Subject(9);
    both.configuration = Configuration(9);

    assert_ne!(both.Digest(), Base().Digest());
    assert_ne!(both.Digest(), Varied(Component::Subject).Digest());
    assert_ne!(both.Digest(), Varied(Component::Configuration).Digest());
}

/// The property that closed OD-ANALYSIS-001.
///
/// A workspace state is not part of what a fact is. The same subject, the same content and
/// the same provider under the same guarantee is one fact whichever tree it was read from.
/// While a key carried a snapshot, it was two — so a corpus recomputed whenever any one
/// file anywhere in the workspace changed, because a workspace snapshot is a digest over
/// all of its members.
#[test]
fn Test_Two_Workspace_States_Should_Not_Produce_Two_Facts()
{
    let key = Base();
    let mut store = MemoryFactStore::New();
    let measured = Fact_From(&key, Snapshot(2));
    let asked_again = Fact_From(&key, Snapshot(9));

    assert_eq!(
        measured.Key().Digest(),
        asked_again.Key().Digest(),
        "the tree reached the identity, so one file's fact is re-addressed by a change to \
         another file entirely"
    );
    store.Materialize(measured, &[]).expect("materializes");
    let served = store
        .Current(&asked_again.identity, GenerationId::INITIAL)
        .expect("the store holds the answer to the question the second one asks");

    assert_eq!(
        served.snapshot,
        Snapshot(2),
        "and what it serves still names the tree it was read from. A reused fact is not a \
         repeated observation, so its provenance must not be restamped"
    );
}
