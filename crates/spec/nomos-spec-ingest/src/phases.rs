use crate::catalog_report::CatalogReport;
use crate::catalog_entity::CatalogEntity;
use crate::statement_divergence::StatementDivergence;
use crate::recorded_statement::RecordedStatement;
use crate::statement_report::StatementReport;
use crate::statement_file::StatementFile;
use nomos_spec_model::{ContentHash, Is_Normalized, Segment};
use nomos_spec_store::{SpecificationStore, StoreError};

#[derive(Debug)]
pub enum IngestError
{
    Store(StoreError),
    Parse(String),
    /// The I1 gate found a disagreement. Ingest stops here.
    GateFailed
    {
        summary: String,
        first: Vec<String>,
    },
}

impl core::fmt::Display for IngestError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Store(error) => write!(formatter, "{error}"),
            Self::Parse(cause) => write!(formatter, "parse: {cause}"),
            Self::GateFailed { summary, first } => write!(
                formatter,
                "the source-truth gate failed ({summary}). Our segmentation disagrees with \
                 v14's manifest, so every hash computed downstream is untrustworthy:\n  {}",
                first.join("\n  ")
            ),
        };
    }
}

impl std::error::Error for IngestError {}

impl From<StoreError> for IngestError
{
    fn from(error: StoreError) -> Self
    {
        return Self::Store(error);
    }
}

/// I0 — store a byte-stream by content.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Blob(store: &mut SpecificationStore, content: &[u8]) -> Result<i64, IngestError>
{
    return Ok(store.Put_Blob(content)?);
}

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

/// # Errors
///
/// Returns [`IngestError::Parse`] if the file cannot be read as the expected shape.
pub fn Parse_Statements(yaml: &str) -> Result<StatementFile, IngestError>
{
    return serde_yaml_ng::from_str(yaml)
        .map_err(|error| IngestError::Parse(format!("statements: {error}")));
}

/// I2 — normative statements, preserving the recorded hash and recomputing it.
///
/// Both are kept. Preserving alone would carry a wrong hash forward unnoticed;
/// recomputing alone would silently redefine identity for every statement in the corpus.
/// Storing both and comparing is what makes a disagreement visible.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Statements(
    store: &mut SpecificationStore,
    file: &StatementFile,
) -> Result<StatementReport, IngestError>
{
    let mut report = StatementReport::default();

    for statement in &file.statements
    {
        Note_Divergence(statement, &mut report);
        Store_Statement(store, statement)?;
        report.ingested = report.ingested.saturating_add(1);
    }

    return Ok(report);
}

/// Records what the recomputed hash says about the recorded one.
///
/// Whether the text is canonical is reported alongside, because a hash that disagrees over
/// text that was never normalized is a different finding from one that disagrees over text
/// that was: the first is a stale recording, the second a changed definition.
fn Note_Divergence(statement: &RecordedStatement, report: &mut StatementReport)
{
    let recomputed = ContentHash::Of(&statement.canonical_text);
    let canonical = Is_Normalized(&statement.canonical_text);

    if recomputed.As_Str() != statement.canonical_hash
    {
        report.divergences.push(StatementDivergence {
            id: statement.id.clone(),
            recorded: statement.canonical_hash.clone(),
            recomputed: recomputed.As_Str().to_owned(),
            text_is_canonical: canonical,
        });
    }

    if !canonical
    {
        report.non_canonical_text.push(statement.id.clone());
    }
}

/// Writes the statement's node and the recorded text hanging off it.
fn Store_Statement(
    store: &mut SpecificationStore,
    statement: &RecordedStatement,
) -> Result<(), IngestError>
{
    let node_uid = store.Upsert_Node(
        &statement.id,
        &statement.kind,
        "canonical",
        "record",
        &statement.source_document,
    )?;

    return Store_Text(store, statement, node_uid);
}

/// The recorded text and the hash it was recorded under, both as the manifest gave them.
///
/// `OR REPLACE` rather than `OR IGNORE`, because re-ingesting a corrected manifest is meant
/// to correct the store — a statement whose text was fixed upstream must not keep the old
/// text just because its identifier is unchanged.
fn Store_Text(
    store: &mut SpecificationStore,
    statement: &RecordedStatement,
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
                statement.kind,
                statement.canonical_text,
                statement.canonical_hash,
            ],
        )
        .map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())))?;

    return Ok(());
}

/// # Errors
///
/// Returns [`IngestError::Parse`] if the catalog is not a JSON array of entities.
pub fn Parse_Catalog(json: &str) -> Result<Vec<CatalogEntity>, IngestError>
{
    return serde_json::from_str(json)
        .map_err(|error| IngestError::Parse(format!("catalog: {error}")));
}

/// Points every name an entity answers to at its node.
fn Record_Aliases(
    store: &mut SpecificationStore,
    entity: &CatalogEntity,
    node_uid: i64,
    report: &mut CatalogReport,
) -> Result<(), IngestError>
{
    for alias in &entity.aliases
    {
        store
            .Connection()
            .execute(
                "INSERT OR IGNORE INTO node_aliases (alias, node_uid) VALUES (?1, ?2)",
                rusqlite::params![alias, node_uid],
            )
            .map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())))?;
        report.aliases = report.aliases.saturating_add(1);
    }

    return Ok(());
}

/// I3 — the node graph.
///
/// # Errors
///
/// Returns [`IngestError`] on any store failure.
pub fn Ingest_Catalog(
    store: &mut SpecificationStore,
    entities: &[CatalogEntity],
) -> Result<CatalogReport, IngestError>
{
    let mut report = CatalogReport::default();

    for entity in entities
    {
        let node_uid = store.Upsert_Node(
            &entity.id,
            &entity.kind,
            if entity.authority.is_empty() { "unstated" } else { &entity.authority },
            "record",
            &entity.title,
        )?;
        report.nodes = report.nodes.saturating_add(1);
        Record_Aliases(store, entity, node_uid, &mut report)?;
    }

    return Ok(report);
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Store() -> SpecificationStore
    {
        return SpecificationStore::In_Memory().expect("opens");
    }

    fn Statements(text: &str, hash: &str) -> StatementFile
    {
        return StatementFile {
            statements: vec![RecordedStatement {
                id: "AGT-001".to_owned(),
                kind: "Requirement".to_owned(),
                canonical_text: text.to_owned(),
                canonical_hash: hash.to_owned(),
                source_document: "a.md".to_owned(),
            }],
        };
    }

    #[test]
    fn Test_A_Matching_Statement_Should_Ingest_Cleanly()
    {
        let text = "Nomos shall do the thing.";
        let file = Statements(text, ContentHash::Of(text).As_Str());

        let report = Ingest_Statements(&mut Store(), &file).expect("ingests");

        assert!(report.Passed());
        assert_eq!(report.ingested, 1);
    }

    /// A recorded hash that disagrees with its own recorded text is the corpus telling
    /// us something is wrong. It must be reported per identifier, never summed.
    #[test]
    fn Test_A_Divergent_Hash_Should_Be_Reported_By_Id()
    {
        let file = Statements("Nomos shall do the thing.", "sha256:0000");

        let report = Ingest_Statements(&mut Store(), &file).expect("ingests");

        assert!(!report.Passed());
        assert_eq!(
            report.divergences.first().map(|d| d.id.as_str()),
            Some("AGT-001")
        );
    }

    /// Text that is not a fixed point of the normalizer makes `canonical_hash` depend on
    /// spacing, so two spellings of one statement become two statements.
    #[test]
    fn Test_Non_Canonical_Text_Should_Be_Reported()
    {
        let text = "Nomos  shall\ndo the thing.";
        let file = Statements(text, ContentHash::Of(text).As_Str());

        let report = Ingest_Statements(&mut Store(), &file).expect("ingests");

        assert!(!report.Passed(), "the hash matches but the text is not canonical");
        assert!(report.divergences.is_empty());
        assert_eq!(report.non_canonical_text, vec!["AGT-001".to_owned()]);
    }

    #[test]
    fn Test_An_Empty_Statement_File_Should_Not_Pass()
    {
        let report = Ingest_Statements(&mut Store(), &StatementFile { statements: Vec::new() })
            .expect("ingests");

        assert!(!report.Passed(), "ingesting nothing is not a clean ingest");
    }

    #[test]
    fn Test_Ingest_Should_Be_Idempotent()
    {
        let mut store = Store();
        let markdown = "# Title\n\nOne.\n";

        let first = Ingest_Source_Document(&mut store, "a.md", "v14.36", markdown).expect("first");
        let second = Ingest_Source_Document(&mut store, "a.md", "v14.36", markdown).expect("second");

        assert_eq!(first, second);
        assert_eq!(
            store.Count(nomos_spec_store::Table::SourceBlocks).expect("counts"),
            first
        );
    }

    #[test]
    fn Test_Catalog_Entities_Should_Become_Nodes_With_Aliases()
    {
        let mut store = Store();
        let entities = vec![CatalogEntity {
            id: "AGT-EXEC-001".to_owned(),
            kind: "requirement".to_owned(),
            title: "a requirement".to_owned(),
            authority: "canonical".to_owned(),
            representation: "record".to_owned(),
            aliases: vec!["AGT-010".to_owned()],
        }];

        let report = Ingest_Catalog(&mut store, &entities).expect("ingests");

        assert_eq!(report.nodes, 1);
        assert_eq!(report.aliases, 1);
        assert_eq!(store.Count(nomos_spec_store::Table::Nodes).expect("counts"), 1);
    }
}
