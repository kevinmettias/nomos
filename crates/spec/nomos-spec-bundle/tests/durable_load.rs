//! A bundle reaches a store that has already been seeded.
//!
//! `OD-SPEC-008` makes the committed durable form of the born-structured layer this
//! bundle's text, and `SQLite` a derived index any checkout rebuilds. Import is the only
//! door from that text into a store, and every store this build assembles is seeded with
//! the governing records before anything else runs — so a door that opened only onto an
//! empty store opened onto no store that exists.
//!
//! What replaced the emptiness demand is not a relaxation. Emptiness was a way of
//! guaranteeing that an import is never a merge; disjointness is the same guarantee stated
//! over content, and it refuses two things emptiness never had to name: content the store
//! already holds, and a reference the bundle makes to a row only the store holds.

use nomos_spec_bundle::{Bundle, BundleError, Export, Import_Bundle, Record, Relation};
use nomos_spec_store::{Seed_Governing_Records, SpecificationStore, Table};
use std::collections::BTreeMap;

/// A document that is nobody's governing record, so it collides with nothing seeded.
const FOREIGN: &str = "---\nid: V9\n---\n# Feature intake\n\nA want, before anybody \
                       decided how to answer it.\n\n## Acceptance\n\n| Question | Answer \
                       |\n| --- | --- |\n| Known good | the asker says so |\n";

const FOREIGN_PATH: &str = "volumes/09-intake.md";
const FOREIGN_REVISION: &str = "v14.36";

/// The store this build actually assembles: governing records first, unconditionally.
fn Seeded() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    Seed_Governing_Records(&mut store).expect("seeds the governing records");

    return store;
}

/// A bundle of content the seed does not hold, and which resolves entirely within itself.
fn Foreign_Bundle() -> Bundle
{
    use nomos_spec_model::Segment;

    let mut source = SpecificationStore::In_Memory().expect("opens");
    let document = source
        .Put_Source_Document(FOREIGN_PATH, FOREIGN_REVISION, FOREIGN)
        .expect("stores the document");
    source
        .Put_Source_Blocks(document, &Segment(FOREIGN))
        .expect("stores the blocks");

    return Export(&source).expect("exports");
}

/// The property the whole item exists for.
#[test]
fn Test_A_Bundle_Should_Load_Into_A_Store_That_Already_Holds_The_Governing_Records()
{
    let mut store = Seeded();
    let bundle = Foreign_Bundle();

    let report = Import_Bundle(&mut store, &bundle).expect("a disjoint bundle must load");

    assert_eq!(report.records, bundle.Manifest().records);
    let (found, _) = store
        .Documents_Named(FOREIGN_PATH, None)
        .expect("queries for the loaded document");
    assert!(!found.is_empty(), "the bundle's document is not in the store");
    assert!(
        store.Node_Uid("D-129").expect("queries").is_some(),
        "the seeded governing records must survive the load"
    );
}

/// The load adds and never rewrites. Checked per table as a difference, because a total
/// cannot tell a row that was placed from a row that was already there.
#[test]
fn Test_The_Seeded_Rows_Should_Be_Left_Untouched_By_The_Load()
{
    let mut store = Seeded();
    let bundle = Foreign_Bundle();
    let before: BTreeMap<&str, u32> = Table::All()
        .iter()
        .map(|table| return (table.Name(), store.Count(*table).expect("counts")))
        .collect();

    Import_Bundle(&mut store, &bundle).expect("a disjoint bundle must load");

    for table in Table::All()
    {
        let expected = Expected_After(&before, &bundle, *table);

        assert_eq!(
            store.Count(*table).expect("counts"),
            expected,
            "{} gained something other than exactly the bundle's rows",
            table.Name()
        );
    }
}

/// What one table held before the load, plus exactly the rows the bundle declares for it.
fn Expected_After(before: &BTreeMap<&str, u32>, bundle: &Bundle, table: Table) -> u32
{
    let declared = bundle
        .Manifest()
        .counts
        .get(table.Name())
        .copied()
        .unwrap_or(0);

    return before
        .get(table.Name())
        .copied()
        .expect("every table was counted")
        .saturating_add(declared);
}

/// Refused, and refused naming the row rather than reporting that a table was not empty —
/// which of the two sides holds the colliding content is the reader's next question.
#[test]
fn Test_A_Bundle_Naming_Content_The_Store_Already_Holds_Should_Be_Refused_By_Name()
{
    let bundle = Export(&Seeded()).expect("exports the seeded store");
    let mut store = Seeded();

    let refusal = Import_Bundle(&mut store, &bundle).expect_err("colliding content must be refused");

    let BundleError::Occupied { table, identity } = refusal
    else
    {
        // Unreachable while a bundle re-imported into the store it was exported from is
        // refused as a collision. Any other variant would mean the import stopped for some
        // reason of its own before reaching the occupied row, and the two assertions below
        // would have no table and no identity to check.
        panic!("expected a collision naming what it found, got {refusal}");
    };
    assert!(!table.is_empty(), "the refusal must name the table");
    assert!(!identity.is_empty(), "the refusal must name the row");
}

/// The hazard that only appears once the store may be non-empty.
///
/// On an empty store a reference to something the bundle does not carry resolved to no row
/// and was refused for free. Against a seeded store the same reference would find the
/// seeded row and bind to it, stitching the bundle onto content it never mentioned — which
/// is precisely the merge this import must not perform by accident. So the references are
/// answered from the bundle, not from the database.
#[test]
fn Test_A_Bundle_Referencing_A_Row_Only_The_Store_Holds_Should_Not_Bind_To_It()
{
    let mut store = Seeded();
    assert!(
        store.Node_Uid("OD-SPEC-008").expect("queries").is_some(),
        "the fixture needs a seeded node for the bundle to reach for"
    );

    let stitched = Bundle::New(
        store.Version(),
        vec![Record::Relation(Relation {
            from_node_id: "OD-SPEC-008".to_owned(),
            relation_type: "relates-to".to_owned(),
            to_node_id: "ARC-SPECDB-002".to_owned(),
        })],
    )
    .expect("builds");

    let refusal = Import_Bundle(&mut store, &stitched).expect_err("a dangling reference must be refused");

    assert!(matches!(refusal, BundleError::Unresolved { .. }), "{refusal}");
}
