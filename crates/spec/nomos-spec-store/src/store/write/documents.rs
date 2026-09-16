//! Writing blobs, documents and nodes through a caller's transaction.

use rusqlite::{Connection, OptionalExtension, params};

use nomos_spec_model::ContentHash;

use crate::DocumentPath;
use crate::DocumentRevision;
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

    if let Some(uid) = Existing_Blob_Uid(connection, &digest)?
    {
        return Ok(uid);
    }

    return Insert_Blob(connection, &digest, content);
}

/// The blob already stored under this digest, if content-addressing already has it.
fn Existing_Blob_Uid(connection: &Connection, digest: &ContentHash) -> Result<Option<i64>, StoreError>
{
    return Ok(connection
        .query_row(
            "SELECT uid FROM blobs WHERE sha256 = ?1",
            params![digest.As_String_Slice()],
            |row| row.get(0),
        )
        .optional()?);
}

/// Inserts bytes nobody has stored under this digest yet.
fn Insert_Blob(connection: &Connection, digest: &ContentHash, content: &[u8]) -> Result<i64, StoreError>
{
    connection.execute(
        "INSERT INTO blobs (sha256, byte_length, content) VALUES (?1, ?2, ?3)",
        params![
            digest.As_String_Slice(),
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
    path: DocumentPath<'_>,
    revision: DocumentRevision<'_>,
    content: &str,
) -> Result<i64, StoreError>
{
    let path = path.0;
    let revision = revision.0;
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::SpecificationStore;

    #[test]
    fn Test_Write_Blob_Should_Store_The_Same_Bytes_Once()
    {
        let store = SpecificationStore::In_Memory().expect("In_Memory builds its own schema, so opening touches no file");

        let first = Write_Blob(store.Connection(), b"same bytes")
            .expect("this connection's schema takes the blob row");
        let second = Write_Blob(store.Connection(), b"same bytes")
            .expect("the same bytes hash to the same blob row");
        let other = Write_Blob(store.Connection(), b"different bytes")
            .expect("different bytes mint a second blob row");

        assert_eq!(first, second);
        assert_ne!(first, other);
    }

    #[test]
    fn Test_Write_Source_Document_Should_Be_Idempotent_For_One_Path_And_Revision()
    {
        let store = SpecificationStore::In_Memory().expect("In_Memory builds its own schema, so opening touches no file");

        let first =
            Write_Source_Document(store.Connection(), DocumentPath("a.md"), DocumentRevision("v1"), "one")
                .expect("the path and revision are free, so the insert takes");
        let second =
            Write_Source_Document(store.Connection(), DocumentPath("a.md"), DocumentRevision("v1"), "one")
                .expect("the path and revision are free, so the insert takes");

        assert_eq!(first, second);
    }

    #[test]
    fn Test_Write_Node_Should_Insert_A_New_Node()
    {
        let store = SpecificationStore::In_Memory().expect("In_Memory builds its own schema, so opening touches no file");

        let uid = Write_Node(
            store.Connection(),
            NodeRow {
                node_id: "D-1",
                kind: "decision",
                authority: "canonical-normative-record",
                representation: "record",
                title: "D-1",
            },
        )
        .expect("the node id is free in this store, so the insert mints it");

        assert!(uid > 0);
    }
}
