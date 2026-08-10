//! The commentary view, and the question only it can answer.

use super::{SpecificationStore, IngestError, Sql};

/// Statements whose only preserved source is commentary.
///
/// D-120 as a query. A statement resting on a game plan and nothing else is a statement
/// the specification never made — the note is the reasoning that led to it, not the
/// authority for it. Statements with no preserved lineage at all are not reported here:
/// that is NSV-PRESERVE-006's subject, and answering it twice in two ways is how one
/// finding becomes two.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Statements_Sourced_Only_From_Commentary(
    store: &SpecificationStore,
) -> Result<Vec<String>, IngestError>
{
    let mut statement = Sql(store.Connection().prepare(
        "SELECT s.statement_id FROM normative_statements s
         WHERE EXISTS (
             SELECT 1 FROM lineage l
             WHERE l.target_statement = s.uid AND l.source_block_uid IS NOT NULL
               AND l.disposition IN ('preserved-verbatim', 'preserved-normalized')
               AND l.source_block_uid IN (SELECT block FROM commentary_blocks)
         )
         AND NOT EXISTS (
             SELECT 1 FROM lineage l
             WHERE l.target_statement = s.uid AND l.source_block_uid IS NOT NULL
               AND l.disposition IN ('preserved-verbatim', 'preserved-normalized')
               AND l.source_block_uid NOT IN (SELECT block FROM commentary_blocks)
         )
         ORDER BY s.statement_id",
    ))?;

    let rows = statement.query_map(rusqlite::params![], |row| row.get(0));
    let found = Sql(rows)?;

    return Sql(found.collect());
}

/// The blocks a commentary node owns, as a view the query above reads twice.
///
/// A view rather than a repeated subquery, so the two halves of "some commentary and no
/// other" cannot drift into asking different questions.
const COMMENTARY_BLOCKS: &str = "CREATE TEMP VIEW IF NOT EXISTS commentary_blocks AS
     SELECT DISTINCT l.source_block_uid AS block FROM lineage l
     JOIN nodes n ON n.uid = l.target_node_uid
     WHERE n.authority = 'commentary' AND l.source_block_uid IS NOT NULL";

/// Prepares the view the commentary query reads.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Prepare_Commentary_View(store: &SpecificationStore) -> Result<(), IngestError>
{
    Sql(store.Connection().execute_batch(COMMENTARY_BLOCKS))?;
    return Ok(());
}
