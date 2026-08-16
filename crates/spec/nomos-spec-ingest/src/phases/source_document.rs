//! Putting one source document in, blocks and all.

use super::{SpecificationStore, IngestError, Segment, RecordedStatement, StoreError};

/// I1 — a source document, its blob and its blocks.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Source_Document(
    store: &mut SpecificationStore,
    path: &str,
    revision: &str,
    markdown: &str,
) -> Result<u32, IngestError>
{
    let document_uid = store.Put_Source_Document(path, revision, markdown)?;
    let blocks = Segment(markdown);
    let written = store.Put_Source_Blocks(document_uid, &blocks)?;

    return Ok(u32::try_from(written).unwrap_or(u32::MAX));
}

/// The recorded text and the hash it was recorded under, both as the manifest gave them.
///
/// `OR REPLACE` rather than `OR IGNORE`, because re-ingesting a corrected manifest is meant
/// to correct the store — a statement whose text was fixed upstream must not keep the old
/// text just because its identifier is unchanged.
pub(super) fn Store_Text(
    store: &mut SpecificationStore,
    statement: &RecordedStatement,
    kind: &str,
    node_uid: i64,
) -> Result<(), IngestError>
{
    store
        .Connection()
        .execute(
            "INSERT OR REPLACE INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                node_uid,
                statement.id,
                kind,
                statement.canonical_text,
                statement.canonical_hash,
            ],
        )
        .map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())))?;

    return Ok(());
}
