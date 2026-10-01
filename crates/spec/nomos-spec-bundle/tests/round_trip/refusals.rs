//! The four things a bundle refuses rather than carries quietly.

use crate::populated::Populated;
use nomos_spec_bundle::{Bundle, BundleError, Export, Import_Bundle, Record};
use nomos_spec_store::{SpecificationStore, Table};

/// Import places a bundle beside what a store holds and never merges into it. A store
/// already holding this bundle's own content is the case that would have to merge, so it
/// is refused — and refused naming the row, rather than reporting that some table was not
/// empty, because which content collided is the reader's next question.
#[test]
fn Test_Importing_Into_A_Store_That_Already_Holds_The_Content_Should_Be_Refused()
{
    let bundle = Export(&Populated()).expect("Export ran over the store Populated() filled");
    let mut occupied = Populated();

    let refusal = Import_Bundle(&mut occupied, &bundle).expect_err("a colliding store must be refused");

    assert!(matches!(refusal, BundleError::Occupied { .. }), "{refusal}");
}

/// A reference to something the bundle does not carry is a refusal, never a NULL. A
/// silent NULL is how a lineage row stops pointing at anything while still counting as
/// a row.
#[test]
fn Test_An_Unresolvable_Reference_Should_Be_Refused()
{
    let complete = Export(&Populated()).expect("Export ran over the store Populated() filled");
    let salvaged = Without_The_Workspace_Concept(&complete);

    assert!(
        salvaged.len() < complete.Records().len(),
        "the negative control removed nothing"
    );

    // The exporter's own version, not a literal: this test is about a dangling reference,
    // and pinning the number here makes every migration fail it for the wrong reason.
    let broken = Bundle::New(complete.Header().schema_version, salvaged)
        .expect("each record the fixture passes survives canonical JSON");
    let mut rebuilt = SpecificationStore::In_Memory().expect("In_Memory applies the schema MIGRATIONS");
    let refusal = Import_Bundle(&mut rebuilt, &broken).expect_err("a dangling reference must be refused");

    assert!(matches!(refusal, BundleError::Unresolved { .. }), "{refusal}");
    assert_eq!(
        rebuilt.Count(Table::Blobs).expect("counts"),
        0,
        "a refused import must leave nothing behind"
    );
}

/// Every record but the node a lineage row points at, which is what makes that reference
/// dangle.
fn Without_The_Workspace_Concept(complete: &Bundle) -> Vec<Record>
{
    return complete
        .Records()
        .iter()
        .filter(|record| {
            return !matches!(record, Record::Node(node) if node.node_id == "CON-WORKSPACE-001");
        })
        .cloned()
        .collect();
}

/// The exporter's completeness guard, against the bug it exists for: a join that drops
/// rows. Without it the export would simply be missing a document and say so nowhere.
#[test]
fn Test_A_Row_The_Export_Query_Drops_Should_Fail_The_Export()
{
    let store = Populated();

    Insert_An_Orphan(&store);

    let refusal = Export(&store).expect_err("a dropped row must fail the export");

    assert!(
        matches!(
            refusal,
            BundleError::Incomplete {
                ref table,
                in_store: DOCUMENTS_AFTER_THE_ORPHAN,
                exported: DOCUMENTS_THE_JOIN_RETURNS
            } if table == "source_documents"
        ),
        "{refusal}"
    );
}

/// How many documents the fixture's store holds once the orphan is inserted beside them.
const DOCUMENTS_AFTER_THE_ORPHAN: u32 = 3;

/// How many of them the export's join reaches, which is one fewer than the store holds.
const DOCUMENTS_THE_JOIN_RETURNS: u32 = 2;

/// A document row the export's join will drop, written with the foreign key relaxed because
/// the point is a row the store holds and the query does not return.
fn Insert_An_Orphan(store: &SpecificationStore)
{
    let connection = store.Connection();

    connection
        .pragma_update(None, "foreign_keys", "OFF")
        .expect("relaxes the constraint");
    connection
        .execute(
            "INSERT INTO source_documents (path, revision, blob_uid)
             VALUES ('volumes/99-orphan.md', 'v14.36', 999999)",
            [],
        )
        .expect("inserts an orphan");
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .expect("restores the constraint");
}

/// The guard the row count cannot be: a column joins the schema and nothing exports it.
///
/// Every row still counts on both sides, so `Assert_Complete` passes and the round trip is
/// a fixpoint — the second export drops the same column the first did. Only a column-level
/// check sees it.
#[test]
fn Test_A_Column_The_Export_Does_Not_Carry_Should_Fail_The_Export()
{
    let store = Populated();
    assert!(Export(&store).is_ok(), "the fixture must export cleanly before it is broken");

    store
        .Connection()
        .execute("ALTER TABLE lineage ADD COLUMN note TEXT", [])
        .expect("adds a column");

    let refusal = Export(&store).expect_err("an uncarried column must fail the export");

    assert!(
        matches!(
            refusal,
            BundleError::UncoveredColumn { ref table, ref column }
                if table == "lineage" && column == "note"
        ),
        "{refusal}"
    );
}
