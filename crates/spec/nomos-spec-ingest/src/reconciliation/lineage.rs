pub(crate) mod origin;
pub(crate) mod relocation;
mod section_report;

use crate::RecordedSection;
use crate::SectionLineage;
use crate::IngestError;
use nomos_spec_store::{DocumentPath, DocumentRevision, SpecificationStore, StoreError};

pub use section_report::SectionReport;

/// # Errors
///
/// Returns [`IngestError::Parse`] if the file is not section lineage.
pub fn Parse_Section_Lineage(yaml: &str) -> Result<SectionLineage, IngestError>
{
    return serde_yaml_ng::from_str(yaml)
        .map_err(|error| IngestError::Parse(format!("section lineage: {error}")));
}

/// I2 — headings and their dispositions.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Section_Lineage(
    store: &mut SpecificationStore,
    lineage: &SectionLineage,
    revision: &str,
) -> Result<SectionReport, IngestError>
{
    let mut report = SectionReport::default();

    for (ordinal, section) in lineage.sections.iter().enumerate()
    {
        let Some(document_uid) =
            Document_Uid(store, DocumentPath(&section.source_document), DocumentRevision(revision))
        else
        {
            report
                .unknown_documents
                .push(format!("{} ({})", section.source_document, section.source_heading));
            continue;
        };

        let at = ordinal;
        Record_Section(store, section, HeadingAt { document_uid, ordinal: at }, &mut report)?;
    }

    return Ok(report);
}

/// Where a heading sits: which document row, and which heading of it.
///
/// The ordinal is the manifest's own order rather than anything recomputed, because the
/// manifest is what the disposition was written against.
#[derive(Clone, Copy)]
struct HeadingAt
{
    document_uid: i64,
    ordinal: usize,
}

/// The document row a path and revision name, if the store holds one.
fn Document_Uid(store: &mut SpecificationStore, path: DocumentPath<'_>, revision: DocumentRevision<'_>) -> Option<i64>
{
    return store
        .Connection()
        .query_row(
            "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
            rusqlite::params![path.0, revision.0],
            |row| row.get(0),
        )
        .ok();
}

/// Writes one heading and the disposition hanging off it.
fn Record_Section(
    store: &mut SpecificationStore,
    section: &RecordedSection,
    at: HeadingAt,
    report: &mut SectionReport,
) -> Result<(), IngestError>
{
    let heading_uid = Upsert_Heading(store, section, at)?;
    report.headings = report.headings.saturating_add(1);

    let inserted_lineage = store.Connection().execute(
        "INSERT OR IGNORE INTO lineage (source_heading_uid, disposition) VALUES (?1, ?2)",
        rusqlite::params![heading_uid, section.disposition],
    );
    Wrap_Sql_Result(inserted_lineage)?;
    report.lineage_rows = report.lineage_rows.saturating_add(1);

    return Ok(());
}

/// The heading row, inserted if it is not already there, and read back either way.
///
/// Read back rather than taking the insert's row id, because `OR IGNORE` gives no row id
/// for a heading a previous ingest already wrote and re-ingesting is meant to be a no-op.
fn Upsert_Heading(
    store: &mut SpecificationStore,
    section: &RecordedSection,
    at: HeadingAt,
) -> Result<i64, IngestError>
{
    let ordinal = i64::try_from(at.ordinal).unwrap_or(i64::MAX);
    let inserted = store.Connection().execute(
        "INSERT OR IGNORE INTO source_headings (document_uid, ordinal, depth, title)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            at.document_uid,
            ordinal,
            section.heading_level,
            section.source_heading
        ],
    );
    Wrap_Sql_Result(inserted)?;

    let selected = store.Connection().query_row(
        "SELECT uid FROM source_headings
         WHERE document_uid = ?1 AND title = ?2 AND depth = ?3",
        rusqlite::params![at.document_uid, section.source_heading, section.heading_level],
        |row| row.get(0),
    );

    return Wrap_Sql_Result(selected);
}

/// Records a disposition for every source block of a document.
///
/// v14's block manifest carries one per block; this writes them so NSV-PRESERVE-002 has
/// something to check.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Block_Dispositions<'a>(
    store: &mut SpecificationStore,
    document: impl Into<DocumentPath<'a>>,
    revision: impl Into<DocumentRevision<'a>>,
    dispositions: &[(u32, String)],
) -> Result<u32, IngestError>
{
    let document = document.into().0;
    let revision = revision.into().0;
    let selected_document = store.Connection().query_row(
        "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
        rusqlite::params![document, revision],
        |row| row.get(0),
    );
    let document_uid: i64 = Wrap_Sql_Result(selected_document)?;

    let mut written = 0_u32;
    for (ordinal, disposition) in dispositions
    {
        if Record_Block(store, document_uid, *ordinal, disposition)?
        {
            written = written.saturating_add(1);
        }
    }

    return Ok(written);
}

/// Records one block's disposition, saying whether there was a block to record it against.
///
/// A manifest ordinal the store does not hold is skipped rather than refused: the manifest
/// covers revisions this store may only hold part of, and a missing block is that rather
/// than a corruption.
fn Record_Block(
    store: &mut SpecificationStore,
    document_uid: i64,
    ordinal: u32,
    disposition: &str,
) -> Result<bool, IngestError>
{
    let found: Option<i64> = store
        .Connection()
        .query_row(
            "SELECT uid FROM source_blocks WHERE document_uid = ?1 AND ordinal = ?2",
            rusqlite::params![document_uid, ordinal],
            |row| row.get(0),
        )
        .ok();

    let Some(block_uid) = found
    else
    {
        return Ok(false);
    };

    let inserted = store.Connection().execute(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition) VALUES (?1, ?2)",
        rusqlite::params![block_uid, disposition],
    );
    Wrap_Sql_Result(inserted)?;

    return Ok(true);
}

fn Wrap_Sql_Result<Value>(result: rusqlite::Result<Value>) -> Result<Value, IngestError>
{
    return result.map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Ingest_Source_Document;

    fn Prepared() -> SpecificationStore
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        Ingest_Source_Document(&mut store, "a.md", "v14.36", "# Title\n\nOne.\n").expect("ingests");
        return store;
    }

    const SECTIONS: &str = "sections:\n\
        - source_document: a.md\n  source_heading: Title\n  heading_level: 1\n  \
        disposition: preserved-or-referenced\n";

    #[test]
    fn Test_Sections_Should_Become_Headings_With_Lineage()
    {
        let mut store = Prepared();
        let lineage = Parse_Section_Lineage(SECTIONS).expect("parses");

        let report = Ingest_Section_Lineage(&mut store, &lineage, "v14.36").expect("ingests");

        assert!(report.Is_Passed());
        assert_eq!(report.headings, 1);
        assert_eq!(report.lineage_rows, 1);
    }

    /// A section naming a document that was never ingested is a gap in the corpus, and
    /// must be named rather than dropped.
    #[test]
    fn Test_A_Section_For_An_Unknown_Document_Should_Be_Reported()
    {
        let mut store = Prepared();
        let lineage = Parse_Section_Lineage(
            "sections:\n- source_document: missing.md\n  source_heading: X\n  \
             heading_level: 1\n  disposition: preserved-or-referenced\n",
        )
        .expect("parses");

        let report = Ingest_Section_Lineage(&mut store, &lineage, "v14.36").expect("ingests");

        assert!(!report.Is_Passed());
        assert_eq!(report.unknown_documents.len(), 1);
    }

    #[test]
    fn Test_Section_Ingest_Should_Be_Idempotent()
    {
        let mut store = Prepared();
        let lineage = Parse_Section_Lineage(SECTIONS).expect("parses");

        Ingest_Section_Lineage(&mut store, &lineage, "v14.36").expect("first");
        Ingest_Section_Lineage(&mut store, &lineage, "v14.36").expect("second");

        assert_eq!(
            store.Count(nomos_spec_store::Table::SourceHeadings).expect("counts"),
            1
        );
        assert_eq!(store.Count(nomos_spec_store::Table::Lineage).expect("counts"), 1);
    }
}
