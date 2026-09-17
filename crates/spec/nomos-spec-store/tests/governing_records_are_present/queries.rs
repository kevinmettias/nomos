//! A seeded store, and the two shapes of query every assertion here reads it with.

use nomos_spec_store::{Seed_Governing_Records, SpecificationStore};

/// The node identifier a query is scoped to.
///
/// Named rather than left a second `&str`, because it sits beside the SQL at every `Column_For_Node`
/// call site and a transposed pair would query the wrong node while still building.
pub(crate) struct NodeId<'a>(pub(crate) &'a str);

/// One counted answer, for a query that binds nothing.
pub(crate) fn Count_From_Sql(store: &SpecificationStore, sql: &str) -> u32
{
    return store
        .Connection()
        .query_row(sql, [], |row| return row.get(0))
        .expect("In_Memory() applied the schema first, so the table the SQL counts exists");
}

/// One text answer, for a query naming one node as `?1`.
pub(crate) fn Column_For_Node(store: &SpecificationStore, sql: &str, node_id: NodeId<'_>) -> String
{
    return store
        .Connection()
        .query_row(sql, rusqlite::params![node_id.0], |row| return row.get(0))
        .expect("a caller passes a node_id the seed wrote, so this SELECT returns its row");
}

pub(crate) fn Seeded() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory() opens no file and applies this crate's schema in the same call");
    Seed_Governing_Records(&mut store)
        .expect("RECORDS is embedded in this crate, so the seed writes text it already holds");
    return store;
}

pub(crate) fn Title_For_Node(store: &SpecificationStore, node_id: &str) -> Option<String>
{
    return store
        .Connection()
        .query_row(
            "SELECT title FROM nodes WHERE node_id = ?1",
            rusqlite::params![node_id],
            |row| row.get(0),
        )
        .ok();
}
