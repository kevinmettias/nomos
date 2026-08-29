//! Putting the normative statements in, and reporting the ones that disagree with their source.

use super::{NodeRow, StatementFile, IngestError, SpecificationStore, StatementReport, RecordedStatement, ContentHash, Is_Normalized, StatementDivergence, Store_Text};

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
pub(super) fn Note_Divergence(statement: &RecordedStatement, report: &mut StatementReport)
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
pub(super) fn Store_Statement(
    store: &mut SpecificationStore,
    statement: &RecordedStatement,
) -> Result<(), IngestError>
{
    let kind = Normalized_Kind(&statement.kind);
    let node_uid = store.Upsert_Node(NodeRow {
        node_id: &statement.id,
        kind: &kind,
        authority: "canonical",
        representation: "record",
        title: &statement.source_document,
    })?;

    return Store_Text(store, statement, &kind, node_uid);
}

/// The catalog's `kind` vocabulary is lowercase and underscore-separated — `requirement`,
/// `user_story` — because it is machine-generated. The statements file is authored for a
/// human reader instead, as `Requirement` and `User Story`, and a project section's
/// `filter: { kind: "requirement" }` is an exact match against whatever a node's row holds.
/// Storing the statement's own spelling would put one concept behind two spellings, and
/// every section that filters a statement by kind would silently select nothing.
fn Normalized_Kind(kind: &str) -> String
{
    return kind.to_lowercase().replace(' ', "_");
}
