//! A table row keeps its kind, and a concept keeps the row it came from.

use crate::populated::{Populated, Rebuilt_From, Records_In};
use nomos_spec_bundle::{Export, Record};
use nomos_spec_store::{SpecificationStore, Table};

/// The fixture carries a table, and the bundle carries its rows.
///
/// `Test_Every_Table_Should_Be_Exercised` already refuses an empty `source_table_rows`, so
/// this is the half it does not cover: that the rows survive with their kinds intact.
/// Without the kind surviving, the pipe-line count and the content count collapse into
/// one number and OD-SPEC-002's whole point is lost on the far side of a round trip.
#[test]
fn Test_The_Bundle_Should_Carry_Typed_Table_Rows()
{
    let store = Populated();
    let rows = store.Count(Table::SourceTableRows).expect("counts");

    assert!(
        rows >= 3,
        "the fixture no longer holds a table, so the row round trip is not being tested"
    );

    let bundle = Export(&store).expect("exports");
    let exported = Records_In(&bundle, Table::SourceTableRows);
    let rebuilt = Rebuilt_From(&bundle);

    assert_eq!(u32::try_from(exported).unwrap_or(u32::MAX), rows);
    assert_eq!(rebuilt.Count(Table::SourceTableRows).expect("counts"), rows);
    assert_eq!(
        Export(&rebuilt).expect("re-exports").Write().expect("writes"),
        bundle.Write().expect("writes"),
        "the rows did not survive the round trip byte for byte"
    );
    Assert_The_Row_Kinds_Survived(&rebuilt);
}

/// The kinds are what keep the pipe-line count and the content count from collapsing into one
/// number, which is OD-SPEC-002's whole point.
fn Assert_The_Row_Kinds_Survived(rebuilt: &SpecificationStore)
{
    let census = rebuilt
        .Row_Census(nomos_spec_store::RowScope::Everything)
        .expect("takes a census");

    assert_eq!(census.separator, 1, "the row kinds did not survive");
    assert_eq!(census.header, 1, "the header kind did not survive");
    assert_eq!(census.content, 1, "the data kind did not survive");
    assert_ne!(
        census.lines, census.non_separator,
        "the kinds collapsed, so both counts became one number"
    );
}

/// A concept restored from a table row must still trace to that row on the far side.
///
/// If it did not, the bundle — the authority committed to git — would carry the concept
/// and lose what produced it, which is exactly the shape the preservation ledger exists
/// to make impossible.
#[test]
fn Test_A_Lineage_To_A_Table_Row_Should_Survive_The_Round_Trip()
{
    let source = Populated();
    let bundle = Export(&source).expect("exports");
    let carried = bundle
        .Records()
        .iter()
        .filter(|record| {
            return matches!(record, Record::Lineage(lineage) if lineage.source_table_row.is_some());
        })
        .count();
    let rebuilt = Rebuilt_From(&bundle);
    let traced = Row_Behind_The_Concept(&rebuilt);

    assert_eq!(carried, 1, "the fixture's row lineage did not reach the bundle");
    assert!(traced.contains("WorkspaceContext"), "it traced to the wrong row: {traced}");
    assert_eq!(
        Export(&rebuilt).expect("re-exports").Write().expect("writes"),
        bundle.Write().expect("writes")
    );
}

/// The table row the restored concept traces to, which must still be there on the far side.
fn Row_Behind_The_Concept(rebuilt: &SpecificationStore) -> String
{
    return rebuilt
        .Connection()
        .query_row(
            "SELECT r.text FROM lineage l
             JOIN source_table_rows r ON r.uid = l.source_table_row_uid
             JOIN nodes n ON n.uid = l.target_node_uid
             WHERE n.node_id = 'CON-WORKSPACE-001'",
            [],
            |row| row.get(0),
        )
        .expect("the concept traces to no row");
}
