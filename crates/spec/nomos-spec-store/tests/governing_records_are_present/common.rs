//! A seeded store, and the two shapes of query every assertion here reads it with.

use nomos_spec_store::{Seed_Governing_Records, SpecificationStore};

/// One counted answer, for a query that binds nothing.
pub(crate) fn Counted(store: &SpecificationStore, sql: &str) -> u32
{
    return store
        .Connection()
        .query_row(sql, [], |row| return row.get(0))
        .expect("queries");
}

/// One text answer, for a query naming one node as `?1`.
pub(crate) fn Column(store: &SpecificationStore, sql: &str, node_id: &str) -> String
{
    return store
        .Connection()
        .query_row(sql, rusqlite::params![node_id], |row| return row.get(0))
        .expect("queries");
}

pub(crate) fn Seeded() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    Seed_Governing_Records(&mut store).expect("seeds");
    return store;
}

pub(crate) fn Title(store: &SpecificationStore, node_id: &str) -> Option<String>
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
