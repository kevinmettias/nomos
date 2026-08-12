//! Writing content into the schema through a caller's transaction.

use nomos_spec_model::{ContentHash, SourceBlock, TableRow, Table_Defects, Table_Rows};
use rusqlite::{Connection, OptionalExtension, params};

use crate::NodeRow;
use crate::StoreError;

use super::EXTERNAL;

/// Writes blocks and their typed rows through a caller's transaction.
///
/// Free rather than a method, and taking a [`Connection`] rather than the store, because
/// [`rusqlite::Transaction`] dereferences to one: the seed, a re-ingest and an authoring
/// commit all reach this same writer, one of them inside a transaction that spans several
/// documents. A second entry point that knew how to write a block would be a second place
/// that could be wrong about what a block is.
///
/// # Errors
///
/// Returns [`StoreError::Table`] if a block's tables do not each carry exactly one
/// delimiter, and [`StoreError`] on any SQL failure.
pub(crate) fn Write_Source_Blocks(
    connection: &Connection,
    document_uid: i64,
    blocks: &[SourceBlock],
) -> Result<usize, StoreError>
{
    Assert_Tables_Are_Sound(document_uid, blocks)?;
    Insert_Blocks(connection, document_uid, blocks)?;

    let uids = Block_Uids(connection, document_uid)?;

    Insert_Table_Rows(connection, document_uid, blocks, &uids)?;

    return Ok(blocks.len());
}

/// Every table in these blocks is one this store will take.
fn Assert_Tables_Are_Sound(document_uid: i64, blocks: &[SourceBlock]) -> Result<(), StoreError>
{
    for block in blocks
    {
        let defects = Table_Defects(&Table_Rows(block));
        if let Some(defect) = defects.first()
        {
            return Err(StoreError::Table {
                document_uid,
                ordinal: block.ordinal,
                cause: defect.to_string(),
            });
        }
    }

    return Ok(());
}

/// The blocks themselves, updated in place where the document already held one.
///
/// Not `INSERT OR REPLACE`. REPLACE deletes the conflicting row and inserts a new one, which
/// hands the block a new `uid` — and `uid` is what every lineage and omission row points at.
/// Re-ingesting a document would silently renumber its blocks and take their dispositions
/// with them.
fn Insert_Blocks(
    connection: &Connection,
    document_uid: i64,
    blocks: &[SourceBlock],
) -> Result<(), StoreError>
{
    let mut insert = connection.prepare(
        "INSERT INTO source_blocks
         (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(document_uid, ordinal) DO UPDATE SET
             kind = excluded.kind,
             heading_path = excluded.heading_path,
             text = excluded.text,
             content_hash = excluded.content_hash,
             normalized_hash = excluded.normalized_hash",
    )?;
    for block in blocks
    {
        insert.execute(params![
            document_uid,
            block.ordinal,
            crate::record::Kind_Label(block.kind),
            block.heading_path.join(" / "),
            block.text,
            block.Content_Hash().As_Str(),
            block.Normalized_Hash().As_Str(),
        ])?;
    }

    return Ok(());
}

/// Every block's uid in one crossing, indexed by ordinal.
///
/// Asking per block cost one round trip for each row of the document, to learn surrogates the
/// same document already decides as a set.
fn Block_Uids(
    connection: &Connection,
    document_uid: i64,
) -> Result<std::collections::BTreeMap<u32, i64>, StoreError>
{
    let mut every_uid = connection
        .prepare("SELECT ordinal, uid FROM source_blocks WHERE document_uid = ?1")?;
    let found = every_uid.query_map(params![document_uid], |row| {
        return Ok((row.get::<_, u32>(0)?, row.get::<_, i64>(1)?));
    })?;
    let mut block_uids = std::collections::BTreeMap::new();

    for entry in found
    {
        let (ordinal, uid) = entry?;
        block_uids.insert(ordinal, uid);
    }

    return Ok(block_uids);
}

/// The rows of every table these blocks carry, under the block that carries them.
///
/// Same reasoning as [`Insert_Blocks`]: updated in place rather than replaced, so a re-ingest
/// does not hand a row a new uid.
fn Insert_Table_Rows(
    connection: &Connection,
    document_uid: i64,
    blocks: &[SourceBlock],
    uids: &std::collections::BTreeMap<u32, i64>,
) -> Result<(), StoreError>
{
    let mut insert_row = connection.prepare(
        "INSERT INTO source_table_rows
         (source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
          content_hash, normalized_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(source_block_uid, ordinal) DO UPDATE SET
             table_ordinal = excluded.table_ordinal,
             kind = excluded.kind,
             cells_json = excluded.cells_json,
             text = excluded.text,
             content_hash = excluded.content_hash,
             normalized_hash = excluded.normalized_hash",
    )?;
    for block in blocks
    {
        let rows = Table_Rows(block);
        let uid = Block_Uid_Of(uids, document_uid, block)?;

        Insert_Rows_Under(&mut insert_row, uid, &rows)?;
    }

    return Ok(());
}

/// The surrogate the block just inserted landed under.
///
/// [`Insert_Blocks`] put every block in, so a missing ordinal is a broken invariant rather
/// than a row that has not arrived yet, and it says so.
fn Block_Uid_Of(
    uids: &std::collections::BTreeMap<u32, i64>,
    document_uid: i64,
    block: &SourceBlock,
) -> Result<i64, StoreError>
{
    let found = uids.get(&block.ordinal).copied();

    return found.ok_or_else(|| {
        return StoreError::Sql(format!(
            "source block {} of document {document_uid} has no uid after insertion",
            block.ordinal
        ));
    });
}

/// One block's table rows, through a prepared insert.
fn Insert_Rows_Under(
    insert_row: &mut rusqlite::Statement<'_>,
    uid: i64,
    rows: &[TableRow],
) -> Result<(), StoreError>
{
    for row in rows
    {
        let cells = serde_json::to_string(&row.cells)
            .map_err(|error| return StoreError::Sql(error.to_string()))?;

        insert_row.execute(params![
            uid,
            row.ordinal,
            row.table_ordinal,
            row.kind.Label(),
            cells,
            row.text,
            row.Content_Hash().As_Str(),
            row.Normalized_Hash().As_Str(),
        ])?;
    }

    return Ok(());
}

/// Every row a mapped query produced, or the first failure it hit.
pub(crate) fn Collected<T>(
    rows: impl Iterator<Item = rusqlite::Result<T>>,
) -> Result<Vec<T>, StoreError>
{
    let mut collected = Vec::new();
    for row in rows
    {
        collected.push(row?);
    }

    return Ok(collected);
}

/// Writes bytes, addressed by content, through a caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Blob(connection: &Connection, content: &[u8]) -> Result<i64, StoreError>
{
    let digest = ContentHash::Of_Bytes(content);
    let existing: Option<i64> = connection
        .query_row(
            "SELECT uid FROM blobs WHERE sha256 = ?1",
            params![digest.As_Str()],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(uid) = existing
    {
        return Ok(uid);
    }
    connection.execute(
        "INSERT INTO blobs (sha256, byte_length, content) VALUES (?1, ?2, ?3)",
        params![
            digest.As_Str(),
            i64::try_from(content.len()).unwrap_or(i64::MAX),
            content
        ],
    )?;

    return Ok(connection.last_insert_rowid());
}

/// Writes a document and its bytes through a caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Source_Document(
    connection: &Connection,
    path: &str,
    revision: &str,
    content: &str,
) -> Result<i64, StoreError>
{
    let blob_uid = Write_Blob(connection, content.as_bytes())?;

    connection.execute(
        "INSERT OR IGNORE INTO source_documents (path, revision, blob_uid) VALUES (?1, ?2, ?3)",
        params![path, revision, blob_uid],
    )?;

    return Ok(connection.query_row(
        "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
        params![path, revision],
        |row| row.get(0),
    )?);
}

/// Writes a node, upgrading a placeholder but never overwriting a real one, through a
/// caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Node(connection: &Connection, node: NodeRow<'_>) -> Result<i64, StoreError>
{
    let NodeRow {
        node_id,
        kind,
        authority,
        representation,
        title,
    } = node;
    connection.execute(
        "INSERT INTO nodes (node_id, kind, authority, representation, title)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(node_id) DO UPDATE SET
             kind = excluded.kind,
             authority = excluded.authority,
             representation = excluded.representation,
             title = excluded.title
         WHERE nodes.authority = ?6",
        params![node_id, kind, authority, representation, title, EXTERNAL],
    )?;

    return Ok(connection.query_row(
        "SELECT uid FROM nodes WHERE node_id = ?1",
        params![node_id],
        |row| row.get(0),
    )?);
}

/// Records an edge and its inverse through a caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Relation(
    connection: &Connection,
    from_node_id: &str,
    relation_type: &str,
    to_node_id: &str,
) -> Result<(), StoreError>
{
    connection.execute(
        "INSERT OR IGNORE INTO relations (from_node_uid, relation_type, to_node_uid)
         SELECT f.uid, ?2, t.uid FROM nodes f, nodes t
         WHERE f.node_id = ?1 AND t.node_id = ?3",
        params![from_node_id, relation_type, to_node_id],
    )?;

    if let Some(inverse) = Inverse_Of(connection, relation_type)?
    {
        connection.execute(
            "INSERT OR IGNORE INTO relations (from_node_uid, relation_type, to_node_uid)
             SELECT f.uid, ?2, t.uid FROM nodes f, nodes t
             WHERE f.node_id = ?1 AND t.node_id = ?3",
            params![to_node_id, inverse, from_node_id],
        )?;
    }

    return Ok(());
}

/// The inverse a relation type declares, if it declares one.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Inverse_Of(
    connection: &Connection,
    relation_type: &str,
) -> Result<Option<String>, StoreError>
{
    return Ok(connection
        .query_row(
            "SELECT inverse_of FROM relation_types WHERE name = ?1",
            params![relation_type],
            |row| row.get(0),
        )
        .optional()?
        .flatten());
}
