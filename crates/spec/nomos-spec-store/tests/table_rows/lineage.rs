//! A row's identity, and what points back at it.
//!
//! The property the restoration exists to establish: one canonical source, two projections.
//! That only holds if a row's `uid` survives re-ingestion, so the renumbering test belongs
//! beside the lineage it would silently break.

use crate::stored::{
    AUTHORED, Blame, Segment, SpecificationStore, Store_Holding_Markdown, TABLE,
    Two_Columns_Of_Row,
};
use nomos_spec_store::{NodeRow, Table};

/// One column of one row, from a query that binds nothing.
fn One_Column_Of_Row<Value: rusqlite::types::FromSql>(
    store: &SpecificationStore,
    sql: &str,
    blame: Blame<'_>,
) -> Value
{
    let found = store.Connection().query_row(sql, [], |row| return row.get(0));

    return found.unwrap_or_else(|cause| panic!("{}: {cause}", blame.0));
}

/// Re-ingest must not renumber rows: `uid` is what a lineage row would point at.
#[test]
fn Test_Rewriting_A_Document_Should_Not_Renumber_Its_Rows()
{
    let mut store =
        SpecificationStore::In_Memory().expect("In_Memory applies the schema in process");
    let document = store
        .Put_Source_Document("doc.md", AUTHORED, TABLE)
        .expect("the fixture path is fresh in this store, so the insert conflicts with nothing");
    let blocks = Segment(TABLE);

    store
        .Put_Source_Blocks(document, &blocks)
        .expect("Put_Source_Blocks inserts every block of the fixture table");
    let before = Row_Uids(&store);
    store.Put_Source_Blocks(document, &blocks).expect("writes again");

    assert!(!before.is_empty(), "no rows, so this test proved nothing");
    assert_eq!(before, Row_Uids(&store), "the rows were reinserted under new uids");
}

fn Row_Uids(store: &SpecificationStore) -> Vec<i64>
{
    return store
        .Connection()
        .prepare("SELECT uid FROM source_table_rows ORDER BY source_block_uid, ordinal")
        .and_then(|mut statement| {
            return statement
                .query_map([], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("the projection names one column the schema declares");
}

/// The property the restoration exists to establish: one canonical source, two
/// projections. A concept minted from a table row resolves back to that row.
#[test]
fn Test_A_Node_Restored_From_A_Row_Should_Trace_To_That_Row()
{
    let store = With_A_Concept_Minted_From_A_Row();
    let traced: String = One_Column_Of_Row(
        &store,
        "SELECT r.text FROM lineage l
         JOIN source_table_rows r ON r.uid = l.source_table_row_uid
         JOIN nodes n ON n.uid = l.target_node_uid
         WHERE n.node_id = 'CON-METRICTRADEOFF-001'",
        Blame("the concept traces to no row"),
    );

    assert!(
        traced.contains("MetricTradeoffProjection"),
        "the concept traced to the wrong row: {traced}"
    );
    assert!(
        !traced.contains("WorkspaceContext"),
        "the concept traced to the whole table, not to its own row"
    );
}

/// The fixture table, with a concept minted from one of its rows and pointed back at it.
fn With_A_Concept_Minted_From_A_Row() -> SpecificationStore
{
    let mut store =
        Store_Holding_Markdown(TABLE).expect("the fixture body is a well-formed table the store takes");
    let document: i64 = One_Column_Of_Row(
        &store,
        "SELECT uid FROM source_documents LIMIT 1",
        Blame("the fixture wrote exactly one source document"),
    );
    let (block_ordinal, row_ordinal): (u32, u32) = Two_Columns_Of_Row(
        &store,
        "SELECT b.ordinal, r.ordinal FROM source_blocks b
         JOIN source_table_rows r ON r.source_block_uid = b.uid
         WHERE r.cells_json LIKE '%MetricTradeoffProjection%'",
        Blame("the fixture's row is the only one naming MetricTradeoffProjection"),
    );
    let row_uid = store
        .Table_Row_Uid(document, block_ordinal, row_ordinal)
        .expect("the fixture's row is addressable at its own block and row ordinals")
        .expect("the row the fixture stored is the one that lookup resolves");
    let node = store
        .Upsert_Node(NodeRow {
            node_id: "CON-METRICTRADEOFF-001",
            kind: "concept",
            authority: "canonical",
            representation: "record",
            title: "MetricTradeoffProjection",
        })
        .expect("mints the concept");
    store
        .Put_Row_Lineage(row_uid, "preserved-verbatim", Some(node))
        .expect("records the lineage");

    return store;
}

/// Idempotence, over the column that is new. Re-running a restoration must not double
/// every concept's lineage, and the unique index is what makes it not.
#[test]
fn Test_Recording_A_Row_Lineage_Twice_Should_Write_One_Row()
{
    let mut store =
        Store_Holding_Markdown(TABLE).expect("the fixture body stores, which is what gives this test rows");
    let row_uid: i64 = One_Column_Of_Row(
        &store,
        "SELECT uid FROM source_table_rows WHERE kind = 'content' LIMIT 1",
        Blame("the fixture table's first content row"),
    );

    store
        .Put_Row_Lineage(row_uid, "preserved-verbatim", None)
        .expect("Put_Row_Lineage accepts the row uid the line above resolved");
    store
        .Put_Row_Lineage(row_uid, "preserved-verbatim", None)
        .expect("records again");

    assert_eq!(store.Count(Table::Lineage).expect("counts"), 1);
}
