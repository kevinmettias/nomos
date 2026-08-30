//! The commentary view, and the question only it can answer.

use super::{SpecificationStore, IngestError, Wrap_Sql_Result};

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
    let mut statement = Wrap_Sql_Result(store.Connection().prepare(
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
    let found = Wrap_Sql_Result(rows)?;

    return Wrap_Sql_Result(found.collect());
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
    Wrap_Sql_Result(store.Connection().execute_batch(COMMENTARY_BLOCKS))?;
    return Ok(());
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_store::NodeRow;

    /// Mints a node, so a test can name a real `target_node_uid` for lineage to point at.
    fn Node(store: &mut SpecificationStore, node_id: &str, authority: &str) -> i64
    {
        return store
            .Upsert_Node(NodeRow {
                node_id,
                kind: "document",
                authority,
                representation: "document",
                title: node_id,
            })
            .expect("mints a node");
    }

    /// A real source block, disposed to `node` by a lineage row of its own — separate from
    /// whatever lineage a test then adds from the same block to a statement.
    fn Block_Disposed_To(store: &mut SpecificationStore, revision: &str, path: &str, node: i64) -> i64
    {
        let text = "# T\n\nBody text.\n";
        let document = store.Put_Source_Document(path, revision, text).expect("puts the document");
        let blocks = nomos_spec_model::Segment(text);
        store.Put_Source_Blocks(document, &blocks).expect("puts the blocks");

        let block: i64 = store
            .Connection()
            .query_row(
                "SELECT uid FROM source_blocks WHERE document_uid = ?1 ORDER BY ordinal LIMIT 1",
                rusqlite::params![document],
                |row| return row.get(0),
            )
            .expect("reads the first block");

        store
            .Connection()
            .execute(
                "INSERT INTO lineage (source_block_uid, disposition, target_node_uid)
                 VALUES (?1, 'preserved-verbatim', ?2)",
                rusqlite::params![block, node],
            )
            .expect("disposes the block to the node");

        return block;
    }

    fn Statement(store: &mut SpecificationStore, node: i64, statement_id: &str) -> i64
    {
        store
            .Connection()
            .execute(
                "INSERT INTO normative_statements
                 (node_uid, statement_id, kind, canonical_text, canonical_hash)
                 VALUES (?1, ?2, 'Requirement', 'Nomos shall.', 'sha256:aa')",
                rusqlite::params![node, statement_id],
            )
            .expect("inserts the statement");

        return store.Connection().last_insert_rowid();
    }

    fn Trace_Statement_To(store: &mut SpecificationStore, block: i64, statement: i64)
    {
        store
            .Connection()
            .execute(
                "INSERT INTO lineage (source_block_uid, disposition, target_statement)
                 VALUES (?1, 'preserved-verbatim', ?2)",
                rusqlite::params![block, statement],
            )
            .expect("traces the statement to the block");
    }

    #[test]
    fn Test_Statements_Sourced_Only_From_Commentary_Should_Report_A_Statement_Resting_Only_On_A_Commentary_Block()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let commentary_node = Node(&mut store, "PLAN-X", "commentary");
        let requirement_node = Node(&mut store, "AGT-100", "canonical");
        let statement = Statement(&mut store, requirement_node, "AGT-100");
        let block = Block_Disposed_To(&mut store, "lineage-notes", "plan.md", commentary_node);
        Trace_Statement_To(&mut store, block, statement);
        Prepare_Commentary_View(&store).expect("prepares");

        let reported = Statements_Sourced_Only_From_Commentary(&store).expect("queries");

        assert_eq!(reported, vec!["AGT-100".to_owned()]);
    }

    #[test]
    fn Test_Prepare_Commentary_View_Should_List_Only_Blocks_Disposed_To_A_Commentary_Node()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let commentary_node = Node(&mut store, "PLAN-Y", "commentary");
        let canonical_node = Node(&mut store, "DOC-Y", "canonical");
        let commentary_block = Block_Disposed_To(&mut store, "lineage-notes", "plan.md", commentary_node);
        let canonical_block = Block_Disposed_To(&mut store, "v14.36", "doc.md", canonical_node);

        Prepare_Commentary_View(&store).expect("prepares");

        let blocks: Vec<i64> = store
            .Connection()
            .prepare("SELECT block FROM commentary_blocks ORDER BY block")
            .expect("prepares the read")
            .query_map([], |row| return row.get(0))
            .expect("queries")
            .collect::<rusqlite::Result<Vec<i64>>>()
            .expect("collects");

        assert_eq!(blocks, vec![commentary_block]);
        assert!(!blocks.contains(&canonical_block));
    }
}
