//! The records that say what the store is are content like any other.

/// The governing records are content like any other, so they must survive the bundle.
/// If they did not, the store committed to git would be missing the records that say
/// what the store is.
#[test]
fn Test_The_Governing_Records_Should_Survive_The_Bundle()
{
    use nomos_spec_bundle::Export;

    let (rebuilt, first) = Rebuilt_From_A_Fresh_Seed();

    Assert_Every_Governing_Record_Survived(&rebuilt);
    assert!(
        rebuilt.Node_Uid("ADR-DOC-001").expect("queries").is_some(),
        "the supersession target did not survive"
    );
    assert_eq!(
        Export(&rebuilt).expect("re-exports").Write().expect("writes"),
        first
    );
}

/// A freshly seeded store, exported and reimported once — the rebuilt store and the bundle
/// text that produced it.
fn Rebuilt_From_A_Fresh_Seed() -> (nomos_spec_store::SpecificationStore, String)
{
    use crate::populated::Reimported;
    use nomos_spec_bundle::Export;
    use nomos_spec_store::SpecificationStore;

    let mut seeded = SpecificationStore::In_Memory().expect("opens");
    nomos_spec_store::Seed_Governing_Records(&mut seeded).expect("seeds");

    let first = Export(&seeded).expect("exports").Write().expect("writes");
    let rebuilt = Reimported(&first);

    return (rebuilt, first);
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
