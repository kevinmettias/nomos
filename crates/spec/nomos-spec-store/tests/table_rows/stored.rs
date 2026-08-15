//! The fixture table every module here is written against, and the two readings of a store.
//!
//! One place to say what a stored table is, so that a test reads as the claim it makes
//! rather than as the setup it needs. A helper reached from one module only stays in that
//! module — `Refusal` with the refusals, `One` with the lineage — because a shared helper
//! is a coupling and four lines of it is not worth one.

// Scoped to this module, not to the whole test binary — an inner attribute on a `mod` file
// reaches no sibling. Whether a fixture here is live is a property of which siblings happen to
// call it, and `main.rs` decides that: `Two` is reached from three modules, `TABLE` from two,
// and dropping a test would make an untouched helper here fail the build in a file nobody
// edited. The repair for that failure is to delete the fixture, which is how a shared fixture
// erodes back into one setup per test — the arrangement this module exists to replace.
#![allow(dead_code)]

pub(crate) use nomos_spec_model::Segment;
pub(crate) use nomos_spec_store::{AUTHORED, RowCensus, RowScope, SpecificationStore, StoreError};

pub(crate) const TABLE: &str = "# Canonical domain model\n\n\
                                | Model | Responsibility |\n\
                                | --- | --- |\n\
                                | WorkspaceContext | Repository, branch, configuration. |\n\
                                | MetricTradeoffProjection | Cost against benefit. |\n";

pub(crate) fn Stored(markdown: &str) -> Result<SpecificationStore, StoreError>
{
    let mut store = SpecificationStore::In_Memory()?;
    let document = store.Put_Source_Document("doc.md", AUTHORED, markdown)?;
    store.Put_Source_Blocks(document, &Segment(markdown))?;
    return Ok(store);
}

pub(crate) fn Census(store: &SpecificationStore, scope: RowScope) -> RowCensus
{
    return store.Row_Census(scope).expect("takes a census");
}

/// Two columns of one row, which is what it takes to address a table row.
pub(crate) fn Two<A: rusqlite::types::FromSql, B: rusqlite::types::FromSql>(
    store: &SpecificationStore,
    sql: &str,
    blame: &str,
) -> (A, B)
{
    return store
        .Connection()
        .query_row(sql, [], |row| return Ok((row.get(0)?, row.get(1)?)))
        .expect(blame);
}
