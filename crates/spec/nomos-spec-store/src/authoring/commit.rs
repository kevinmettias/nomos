//! Applying a previewed edit to the store, as one transaction.

use nomos_spec_model::{
    RecordFrontMatter, RecordRelation, Segment, SourceBlock,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::commit_report::CommitReport;
use crate::edit_error::EditError;
use crate::edit_preview::EditPreview;
use crate::node_row::NodeRow;
use crate::store::{
    EXTERNAL, Inverse_Of, Write_Blob, Write_Node, Write_Relation,
    Write_Source_Blocks,
};
use crate::store_error::StoreError;


use super::write::{
    Dispose_Blocks, Write_Declared_Relations, Write_Front_Matter, Write_Headings,
};

/// Writes a previewed edit through a caller's transaction.
pub(super) fn Apply(connection: &Connection, preview: &EditPreview) -> Result<CommitReport, EditError>
{
    let claimed = &preview.staged.claimed;
    let document_uid = claimed.document_uid;
    let front_matter = &preview.staged.record.front_matter;

    if let Some((_, after)) = preview.Rename()
    {
        Rename_Document(connection, document_uid, after, &claimed.projection.revision)?;
    }
    Rewrite_Bytes(connection, document_uid, &preview.staged.markdown)?;
    let node_uid = Restate_Node(connection, front_matter)?;
    let blocks = Segment(&preview.staged.record.body);
    let blocks_removed = Rewrite_Blocks(connection, document_uid, node_uid, &blocks)?;
    Write_Front_Matter(connection, document_uid, node_uid, front_matter)?;
    Write_Declared_Relations(connection, document_uid, &front_matter.relations)?;
    Update_Graph(connection, &front_matter.id, preview)?;

    return Ok(Reported(preview, blocks.len(), blocks_removed));
}

/// Replaces the document's stored bytes with the ones the author staged.
fn Rewrite_Bytes(connection: &Connection, document_uid: i64, markdown: &str)
    -> Result<(), StoreError>
{
    let blob_uid = Write_Blob(connection, markdown.as_bytes())?;

    connection.execute(
        "UPDATE source_documents SET blob_uid = ?2 WHERE uid = ?1",
        params![document_uid, blob_uid],
    )?;

    return Ok(());
}

/// Moves the title and the kind with the author's own edit.
///
/// `Write_Node` refuses to overwrite a real node deliberately — the first writer of a record
/// is its author and a later ingest pass must not restate it — and this is that author,
/// arriving through the door the record decided they would use.
fn Restate_Node(connection: &Connection, front_matter: &RecordFrontMatter)
    -> Result<i64, StoreError>
{
    let node_uid = Node_Uid_Of(connection, &front_matter.id)?;

    connection.execute(
        "UPDATE nodes SET kind = ?2, authority = ?3, title = ?4 WHERE uid = ?1",
        params![
            node_uid,
            front_matter.kind,
            front_matter.authority,
            front_matter.title
        ],
    )?;

    return Ok(node_uid);
}

/// Replaces the document's blocks with the staged ones, and reports how many the edit
/// shortened it past.
fn Rewrite_Blocks(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    blocks: &[SourceBlock],
) -> Result<usize, EditError>
{
    Write_Source_Blocks(connection, document_uid, blocks)?;
    Write_Headings(connection, document_uid, node_uid, blocks)?;
    Dispose_Blocks(connection, document_uid, node_uid, blocks)?;

    return Prune_Blocks_Beyond(connection, document_uid, blocks.len());
}

/// What the commit did, as the caller reads it back.
fn Reported(preview: &EditPreview, blocks: usize, blocks_removed: usize) -> CommitReport
{
    return CommitReport {
        node_id: preview.staged.record.front_matter.id.clone(),
        path: preview.staged.path.clone(),
        blocks,
        blocks_removed,
        relations_added: preview.relations_added.len(),
        relations_removed: preview.relations_removed.len(),
        renamed: preview.Rename().is_some(),
    };
}

/// Moves a document to a new path, keeping its surrogate.
///
/// `UPDATE` rather than insert-and-delete, because `uid` is what every block, lineage and
/// omission row hangs from. A rename that minted a new document row would be a rename that
/// silently dropped the record's history — which is what makes a rename a migration
/// elsewhere and an ordinary edit here.
fn Rename_Document(
    connection: &Connection,
    document_uid: i64,
    path: &str,
    revision: &str,
) -> Result<(), EditError>
{
    let holder = Document_At(connection, path, revision)?;

    if holder.is_some_and(|found| return found != document_uid)
    {
        return Err(EditError::PathTaken {
            path: path.to_owned(),
        });
    }

    connection
        .execute(
            "UPDATE source_documents SET path = ?2 WHERE uid = ?1",
            params![document_uid, path],
        )
        .map_err(StoreError::from)?;

    return Ok(());
}

/// Which document, if any, already lives at this address.
fn Document_At(connection: &Connection, path: &str, revision: &str)
    -> Result<Option<i64>, StoreError>
{
    return Ok(connection
        .query_row(
            "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
            params![path, revision],
            |row| row.get(0),
        )
        .optional()?);
}

/// Removes the blocks an edit shortened the document past.
///
/// Refuses rather than deletes where a block carries a justified omission. An omission row
/// is the only sanctioned way for content to leave this store, carrying its reason and the
/// decision that allowed it; deleting the block underneath it would destroy the justification
/// and leave the content gone — the exact pair of events this schema exists to prevent.
fn Prune_Blocks_Beyond(
    connection: &Connection,
    document_uid: i64,
    keep: usize,
) -> Result<usize, EditError>
{
    let keep = i64::try_from(keep).unwrap_or(i64::MAX);
    let justified = Justified_Omissions_Beyond(connection, document_uid, keep)?;

    if justified > 0
    {
        return Err(EditError::Store(StoreError::Sql(format!(
            "{justified} block(s) this edit removes carry a justified omission; deleting them \
             would destroy the justification and the content together"
        ))));
    }

    Detach_Blocks_Beyond(connection, document_uid, keep)?;

    let removed = connection
        .execute(
            "DELETE FROM source_blocks WHERE document_uid = ?1 AND ordinal > ?2",
            params![document_uid, keep],
        )
        .map_err(StoreError::from)?;

    return Ok(removed);
}

/// How many blocks past `keep` carry a justified omission.
fn Justified_Omissions_Beyond(connection: &Connection, document_uid: i64, keep: i64)
    -> Result<u32, StoreError>
{
    return Ok(connection.query_row(
        "SELECT count(*) FROM omissions o
         JOIN source_blocks b ON b.uid = o.source_block_uid
         WHERE b.document_uid = ?1 AND b.ordinal > ?2",
        params![document_uid, keep],
        |row| row.get(0),
    )?);
}

/// Removes everything hanging from the blocks past `keep`, so the blocks themselves can go.
///
/// Written out rather than iterated over a list. These are three different statements run
/// once each, in an order the foreign keys require — a row's lineage before the row, and both
/// before the block they hang from — and a loop said "repeat this" about three steps that are
/// not repetitions of one another.
fn Detach_Blocks_Beyond(connection: &Connection, document_uid: i64, keep: i64)
    -> Result<(), StoreError>
{
    connection.execute(
        "DELETE FROM lineage WHERE source_table_row_uid IN (
             SELECT r.uid FROM source_table_rows r
             JOIN source_blocks b ON b.uid = r.source_block_uid
             WHERE b.document_uid = ?1 AND b.ordinal > ?2)",
        params![document_uid, keep],
    )?;
    connection.execute(
        "DELETE FROM lineage WHERE source_block_uid IN (
             SELECT uid FROM source_blocks WHERE document_uid = ?1 AND ordinal > ?2)",
        params![document_uid, keep],
    )?;
    connection.execute(
        "DELETE FROM source_table_rows WHERE source_block_uid IN (
             SELECT uid FROM source_blocks WHERE document_uid = ?1 AND ordinal > ?2)",
        params![document_uid, keep],
    )?;

    return Ok(());
}

/// Brings the graph into line with what the record now declares.
fn Update_Graph(
    connection: &Connection,
    node_id: &str,
    preview: &EditPreview,
) -> Result<(), StoreError>
{
    for relation in &preview.relations_added
    {
        Add_Relation(connection, node_id, relation)?;
    }
    for relation in &preview.relations_removed
    {
        Remove_Relation(connection, node_id, relation)?;
    }

    return Ok(());
}

/// Declares a relation, minting the target as an external node when nothing holds it yet.
fn Add_Relation(connection: &Connection, node_id: &str, relation: &RecordRelation)
    -> Result<(), StoreError>
{
    if Optional_Node_Uid(connection, &relation.target)?.is_none()
    {
        Write_Node(connection, NodeRow {
            node_id: &relation.target,
            kind: "unknown",
            authority: EXTERNAL,
            representation: "record",
            title: &relation.target,
        })?;
    }

    return Write_Relation(connection, node_id, &relation.relation, &relation.target);
}

/// Withdraws a relation, and the inverse the schema keeps beside it.
fn Remove_Relation(connection: &Connection, node_id: &str, relation: &RecordRelation)
    -> Result<(), StoreError>
{
    Delete_Relation(connection, node_id, &relation.relation, &relation.target)?;

    if let Some(inverse) = Inverse_Of(connection, &relation.relation)?
    {
        Delete_Relation(connection, &relation.target, &inverse, node_id)?;
    }

    return Ok(());
}

fn Delete_Relation(
    connection: &Connection,
    from_node_id: &str,
    relation_type: &str,
    to_node_id: &str,
) -> Result<(), StoreError>
{
    connection.execute(
        "DELETE FROM relations
         WHERE from_node_uid = (SELECT uid FROM nodes WHERE node_id = ?1)
           AND relation_type = ?2
           AND to_node_uid = (SELECT uid FROM nodes WHERE node_id = ?3)",
        params![from_node_id, relation_type, to_node_id],
    )?;

    return Ok(());
}

fn Node_Uid_Of(connection: &Connection, node_id: &str) -> Result<i64, StoreError>
{
    return Optional_Node_Uid(connection, node_id)?.ok_or_else(|| {
        return StoreError::Sql(format!(
            "{node_id} was read out of this store and is no longer in it"
        ));
    });
}

fn Optional_Node_Uid(connection: &Connection, node_id: &str) -> Result<Option<i64>, StoreError>
{
    return Ok(connection
        .query_row(
            "SELECT uid FROM nodes WHERE node_id = ?1",
            params![node_id],
            |row| row.get(0),
        )
        .optional()?);
}
