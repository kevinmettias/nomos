//! Writing blobs, documents and nodes through a caller's transaction.

use nomos_spec_model::ContentHash;
use rusqlite::{Connection, OptionalExtension, params};

use crate::NodeRow;
use crate::StoreError;

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
    Upsert_Node_Row(connection, node)?;

    return Node_Uid_By_Id(connection, node.node_id);
}

/// Inserts a node, or upgrades a placeholder — never a real one — through the authority
/// guard on the conflict clause.
fn Upsert_Node_Row(connection: &Connection, node: NodeRow<'_>) -> Result<(), StoreError>
{
    use crate::EXTERNAL;

    let NodeRow { node_id, kind, authority, representation, title } = node;
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

    return Ok(());
}

/// The surrogate a node's identifier resolves to.
fn Node_Uid_By_Id(connection: &Connection, node_id: &str) -> Result<i64, StoreError>
{
    return Ok(connection.query_row(
        "SELECT uid FROM nodes WHERE node_id = ?1",
        params![node_id],
        |row| row.get(0),
    )?);
}
