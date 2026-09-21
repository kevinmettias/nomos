//! The records that say what the store is are content like any other.

/// The governing records are content like any other, so they must survive the bundle.
/// If they did not, the store committed to git would be missing the records that say
/// what the store is.
#[test]
fn Test_The_Governing_Records_Should_Survive_The_Bundle()
{
    use nomos_spec_bundle::Export;

    let rebuilt = Rebuilt_From_A_Fresh_Seed();

    Assert_Every_Governing_Record_Survived(&rebuilt.store);
    assert!(
        rebuilt.store.Node_Uid("ADR-DOC-001").expect("queries").is_some(),
        "the supersession target did not survive"
    );
    assert_eq!(
        Export(&rebuilt.store)
            .expect("Export ran over the store the import filled")
            .Write()
            .expect("Write serializes the records the bundle carries"),
        rebuilt.text
    );
}

/// A freshly seeded store and the bundle text it was rebuilt from.
///
/// The two are named rather than returned as a bare pair, so a caller cannot read the text
/// as the store or the store as the text.
struct Rebuilt
{
    /// The store the seed was imported back into.
    store: nomos_spec_store::SpecificationStore,
    /// The bundle text that produced it.
    text: String,
}

/// A freshly seeded store, exported and reimported once — the rebuilt store and the bundle
/// text that produced it.
fn Rebuilt_From_A_Fresh_Seed() -> Rebuilt
{
    use crate::populated::Rebuilt_From_Bundle_Text;
    use nomos_spec_bundle::Export;
    use nomos_spec_store::SpecificationStore;

    let mut seeded = SpecificationStore::In_Memory().expect("In_Memory applies the schema MIGRATIONS");
    nomos_spec_store::Seed_Governing_Records(&mut seeded)
        .expect("the seed places every governing record the store declares");

    let first = Export(&seeded)
        .expect("Export ran over the store the seed filled")
        .Write()
        .expect("Write serializes the records the bundle carries");
    let rebuilt = Rebuilt_From_Bundle_Text(&first);

    return Rebuilt {
        store: rebuilt,
        text: first,
    };
}

/// Every governing record the seed declares is present in the rebuilt store.
fn Assert_Every_Governing_Record_Survived(rebuilt: &nomos_spec_store::SpecificationStore)
{
    for id in nomos_spec_store::GOVERNING_RECORD_IDS
    {
        assert!(
            rebuilt.Node_Uid(id).expect("queries").is_some(),
            "{id} did not survive the round trip"
        );
    }
}
