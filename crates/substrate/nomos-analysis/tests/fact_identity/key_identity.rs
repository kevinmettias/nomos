//! What a fact is addressed by: every component reaches the digest, and nothing else does.

use crate::key::{
    Base, Configuration_Id_From_Seed, Key_With_Varied_Component, Memory_Store_For_Key,
    Snapshot_Id_From_Seed, Subject_Id_From_Seed,
};
use nomos_analysis::{Component, FactKey, FactStore, MaterializedFact, MemoryFactStore};
use nomos_contracts::{GenerationId, SnapshotId};

/// The position each `Component` variant occupies in `Component::All()`'s own order. They
/// are named rather than written so an arm that drifts is a name to compare, and so the
/// ordinal carries the meaning it has: this variant's place in the enumerated universe.
const CONTRACT_ORDINAL: usize = 0;
const CONTRACT_VERSION_ORDINAL: usize = 1;
const SUBJECT_ORDINAL: usize = 2;
const SEMANTIC_INPUTS_ORDINAL: usize = 3;
const PROVIDER_ORDINAL: usize = 4;
const PROVIDER_VERSION_ORDINAL: usize = 5;
const GUARANTEE_ORDINAL: usize = 6;
const VARIANT_ORDINAL: usize = 7;
const CONFIGURATION_ORDINAL: usize = 8;

/// The seed the two differing components are set to, matching [`Key_With_Varied_Component`]'s own substitution
/// so a two-component key can be compared against the one-component keys.
const VARIED_SEED: u8 = 9;

/// The two trees the same fact is read from: the one it was measured at, and the one that
/// asks the question again.
const MEASURED_TREE_SEED: u8 = 2;
const ASKING_TREE_SEED: u8 = 9;

/// The same fact, read from a named tree.
fn Fact_From(key: &FactKey, snapshot: SnapshotId) -> MaterializedFact
{
    let mut fact = crate::key::Materialized_Fact_From_Key(key, GenerationId::INITIAL);
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
    fn Ordinal_Of_Component(component: Component) -> usize
    {
        return match component
        {
            Component::Contract => CONTRACT_ORDINAL,
            Component::ContractVersion => CONTRACT_VERSION_ORDINAL,
            Component::Subject => SUBJECT_ORDINAL,
            Component::SemanticInputs => SEMANTIC_INPUTS_ORDINAL,
            Component::Provider => PROVIDER_ORDINAL,
            Component::ProviderVersion => PROVIDER_VERSION_ORDINAL,
            Component::Guarantee => GUARANTEE_ORDINAL,
            Component::Variant => VARIANT_ORDINAL,
            Component::Configuration => CONFIGURATION_ORDINAL,
        };
    }

    for (index, component) in Component::All().iter().enumerate()
    {
        assert_eq!(
            Ordinal_Of_Component(*component),
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
            Key_With_Varied_Component(*component).Digest(),
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
    let store = Memory_Store_For_Key(&Base());

    for component in Component::All()
    {
        let varied = Key_With_Varied_Component(*component);

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
    both.subject = Subject_Id_From_Seed(VARIED_SEED);
    both.configuration = Configuration_Id_From_Seed(VARIED_SEED);

    assert_ne!(both.Digest(), Base().Digest());
    assert_ne!(both.Digest(), Key_With_Varied_Component(Component::Subject).Digest());
    assert_ne!(both.Digest(), Key_With_Varied_Component(Component::Configuration).Digest());
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
    let measured = Fact_From(&key, Snapshot_Id_From_Seed(MEASURED_TREE_SEED));
    let asked_again = Fact_From(&key, Snapshot_Id_From_Seed(ASKING_TREE_SEED));

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
        Snapshot_Id_From_Seed(MEASURED_TREE_SEED),
        "and what it serves still names the tree it was read from. A reused fact is not a \
         repeated observation, so its provenance must not be restamped"
    );
}
