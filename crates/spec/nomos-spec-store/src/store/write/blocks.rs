//! Writing source blocks and their typed table rows through a caller's transaction.

use nomos_spec_model::{SourceBlock, TableRow, Table_Defects, Table_Rows};
use rusqlite::{Connection, params};

use crate::StoreError;

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
            block.Content_Hash().As_String_Slice(),
            block.Normalized_Hash().As_String_Slice(),
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
            row.Content_Hash().As_String_Slice(),
            row.Normalized_Hash().As_String_Slice(),
        ])?;
    }

    return Ok(());
}
