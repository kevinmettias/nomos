use crate::phases::IngestError;
use nomos_spec_store::{SpecificationStore, StoreError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RecordedSection
{
    pub source_document: String,
    pub source_heading: String,
    #[serde(default)]
    pub heading_level: i64,
    pub disposition: String,
    #[serde(default)]
    pub stable_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SectionLineage
{
    pub sections: Vec<RecordedSection>,
}

#[derive(Debug, Default)]
pub struct SectionReport
{
    pub headings: u32,
    pub lineage_rows: u32,
    /// Sections naming a document that was never ingested, per section rather than
    /// counted.
    pub unknown_documents: Vec<String>,
}

impl SectionReport
{
    #[must_use]
    pub fn Passed(&self) -> bool
    {
        return self.unknown_documents.is_empty() && self.headings > 0;
    }
}

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
        let document_uid: Option<i64> = store
            .Connection()
            .query_row(
                "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
                rusqlite::params![section.source_document, revision],
                |row| row.get(0),
            )
            .ok();

        let Some(document_uid) = document_uid
        else
        {
            report
                .unknown_documents
                .push(format!("{} ({})", section.source_document, section.source_heading));
            continue;
        };

        let ordinal = i64::try_from(ordinal).unwrap_or(i64::MAX);
        let inserted_heading = store.Connection().execute(
            "INSERT OR IGNORE INTO source_headings (document_uid, ordinal, depth, title)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                document_uid,
                ordinal,
                section.heading_level,
                section.source_heading
            ],
        );
        Sql(inserted_heading)?;

        let selected_heading = store.Connection().query_row(
            "SELECT uid FROM source_headings
             WHERE document_uid = ?1 AND title = ?2 AND depth = ?3",
            rusqlite::params![document_uid, section.source_heading, section.heading_level],
            |row| row.get(0),
        );
        let heading_uid: i64 = Sql(selected_heading)?;
        report.headings = report.headings.saturating_add(1);

        let inserted_lineage = store.Connection().execute(
            "INSERT OR IGNORE INTO lineage (source_heading_uid, disposition) VALUES (?1, ?2)",
            rusqlite::params![heading_uid, section.disposition],
        );
        Sql(inserted_lineage)?;
        report.lineage_rows = report.lineage_rows.saturating_add(1);
    }

    return Ok(report);
}

/// Records a disposition for every source block of a document.
///
/// v14's block manifest carries one per block; this writes them so NSV-PRESERVE-002 has
/// something to check.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Block_Dispositions(
    store: &mut SpecificationStore,
    document: &str,
    revision: &str,
    dispositions: &[(u32, String)],
) -> Result<u32, IngestError>
{
    let selected_document = store.Connection().query_row(
        "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
        rusqlite::params![document, revision],
        |row| row.get(0),
    );
    let document_uid: i64 = Sql(selected_document)?;

    let mut written = 0_u32;
    for (ordinal, disposition) in dispositions
    {
        let block_uid: Option<i64> = store
            .Connection()
            .query_row(
                "SELECT uid FROM source_blocks WHERE document_uid = ?1 AND ordinal = ?2",
                rusqlite::params![document_uid, ordinal],
                |row| row.get(0),
            )
            .ok();

        let Some(block_uid) = block_uid
        else
        {
            continue;
        };

        let inserted_lineage = store.Connection().execute(
            "INSERT OR IGNORE INTO lineage (source_block_uid, disposition) VALUES (?1, ?2)",
            rusqlite::params![block_uid, disposition],
        );
        Sql(inserted_lineage)?;
        written = written.saturating_add(1);
    }

    return Ok(written);
}

fn Sql<T>(result: rusqlite::Result<T>) -> Result<T, IngestError>
{
    return result.map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::phases::Ingest_Source_Document;

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

        assert!(report.Passed());
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

        assert!(!report.Passed());
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
