//! The fixture table every module here is written against, and the two readings of a store.
//!
//! One place to say what a stored table is, so that a test reads as the claim it makes
//! rather than as the setup it needs. A helper reached from one module only stays in that
//! module — `Refusal_Of_Store` with the refusals, `One_Column_Of_Row` with the lineage —
//! because a shared helper is a coupling and four lines of it is not worth one.

// Scoped to this module, not to the whole test binary — an inner attribute on a `mod` file
// reaches no sibling. Whether a fixture here is live is a property of which siblings happen to
// call it, and `main.rs` decides that: `Two_Columns_Of_Row` is reached from three modules, `TABLE` from two,
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

pub(crate) fn Store_Holding_Markdown(markdown: &str) -> Result<SpecificationStore, StoreError>
{
    let mut store = SpecificationStore::In_Memory()?;
    let document = store.Put_Source_Document("doc.md", AUTHORED, markdown)?;
    store.Put_Source_Blocks(document, &Segment(markdown))?;
    return Ok(store);
}

pub(crate) fn Row_Census_For_Scope(store: &SpecificationStore, scope: RowScope) -> RowCensus
{
    return store.Row_Census(scope).expect("takes a census");
}

/// Why a fixture query had to return a row, named so a caller cannot transpose it with the
/// query text it sits beside.
///
/// The two are both prose describing one call, so as two bare `&str`s they compile in either
/// order — and the failure would then quote the other one's explanation at whichever row was
/// missing.
pub(crate) struct Blame<'a>(pub(crate) &'a str);

/// Two columns of one row, which is what it takes to address a table row.
pub(crate) fn Two_Columns_Of_Row<First: rusqlite::types::FromSql, Second: rusqlite::types::FromSql>(
    store: &SpecificationStore,
    sql: &str,
    blame: Blame<'_>,
) -> (First, Second)
{
    let found = store.Connection().query_row(sql, [], |row| return Ok((row.get(0)?, row.get(1)?)));

    return found.unwrap_or_else(|cause| panic!("{}: {cause}", blame.0));
}
