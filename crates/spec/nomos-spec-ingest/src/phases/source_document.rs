//! Putting one source document in, blocks and all.

use super::{SpecificationStore, IngestError, Segment, RecordedStatement, StoreError};
use nomos_spec_store::{DocumentPath, DocumentRevision};

/// I1 — a source document, its blob and its blocks.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Source_Document<'a>(
    store: &mut SpecificationStore,
    path: impl Into<DocumentPath<'a>>,
    revision: impl Into<DocumentRevision<'a>>,
    markdown: &str,
) -> Result<u32, IngestError>
{
    let document_uid = store.Put_Source_Document(path.into(), revision.into(), markdown)?;
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Ingest_Source_Document_Should_Store_The_Document_And_Its_Blocks()
    {
        let mut store = Store();
        let markdown = "# Title\n\nOne.\n";

        let written = Ingest_Source_Document(&mut store, "a.md", "v14.36", markdown).expect("ingests");

        assert_eq!(written, 2, "a heading and a prose block");
        assert_eq!(
            store.Count(nomos_spec_store::Table::SourceBlocks).expect("counts"),
            2
        );
    }

    #[test]
    fn Test_Store_Text_Should_Record_The_Statements_Canonical_Text_And_Hash()
    {
        let mut store = Store();
        let node_uid = store
            .Upsert_Node(nomos_spec_store::NodeRow {
                node_id: "AGT-001",
                kind: "requirement",
                authority: "canonical",
                representation: "record",
                title: "a requirement",
            })
            .expect("upserts");
        let statement = RecordedStatement {
            id: "AGT-001".to_owned(),
            kind: "Requirement".to_owned(),
            canonical_text: "Nomos shall do the thing.".to_owned(),
            canonical_hash: "sha256:deadbeef".to_owned(),
            source_document: "a.md".to_owned(),
        };

        Store_Text(&mut store, &statement, "requirement", node_uid).expect("stores");

        let stored_text: String = store
            .Connection()
            .query_row(
                "SELECT canonical_text FROM normative_statements WHERE statement_id = 'AGT-001'",
                [],
                |row| row.get(0),
            )
            .expect("reads the row Store_Text wrote");
        assert_eq!(stored_text, "Nomos shall do the thing.");
    }

    fn Store() -> SpecificationStore
    {
        return SpecificationStore::In_Memory().expect("opens");
    }
}
