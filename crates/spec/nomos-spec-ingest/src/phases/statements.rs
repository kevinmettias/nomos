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

    if recomputed.As_String_Slice() != statement.canonical_hash
    {
        report.divergences.push(StatementDivergence {
            id: statement.id.clone(),
            recorded: statement.canonical_hash.clone(),
            recomputed: recomputed.As_String_Slice().to_owned(),
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Parse_Statements_Should_Read_A_Yaml_Statement_File()
    {
        let yaml = "statements:\n- id: AGT-001\n  kind: Requirement\n  \
                     canonical_text: Nomos shall do the thing.\n  \
                     canonical_hash: sha256:deadbeef\n  source_document: a.md\n";

        let file = Parse_Statements(yaml).expect("parses");

        assert_eq!(file.statements.len(), 1);
        assert_eq!(file.statements.first().expect("the assertion above confirms exactly one statement").id, "AGT-001");
    }

    #[test]
    fn Test_Ingest_Statements_Should_Report_How_Many_Ingested()
    {
        let text = "Nomos shall do the thing.";
        let file = StatementFile {
            statements: vec![One_Statement("Requirement", text, ContentHash::Of(text).As_String_Slice())],
        };

        let report = Ingest_Statements(&mut Store(), &file).expect("ingests");

        assert_eq!(report.ingested, 1);
        assert!(report.Is_Passing());
    }

    #[test]
    fn Test_Note_Divergence_Should_Flag_A_Recorded_Hash_That_Disagrees()
    {
        let statement = One_Statement("Requirement", "Nomos shall do the thing.", "sha256:0000");
        let mut report = StatementReport::default();

        Note_Divergence(&statement, &mut report);

        assert_eq!(report.divergences.len(), 1);
        assert_eq!(report.divergences.first().expect("the assertion above confirms exactly one divergence").id, "AGT-001");
    }

    #[test]
    fn Test_Store_Statement_Should_Write_A_Node_Under_The_Catalogs_Lowercase_Kind()
    {
        let mut store = Store();
        let statement = One_Statement("User Story", "Nomos shall do the thing.", "sha256:deadbeef");

        Store_Statement(&mut store, &statement).expect("stores");

        let summary = store.Node_Summary("AGT-001").expect("reads").expect("node exists");
        assert_eq!(summary.kind, "user_story");
    }

    fn Store() -> SpecificationStore
    {
        return SpecificationStore::In_Memory().expect("opens");
    }

    fn One_Statement(kind: &str, text: &str, hash: &str) -> RecordedStatement
    {
        return RecordedStatement {
            id: "AGT-001".to_owned(),
            kind: kind.to_owned(),
            canonical_text: text.to_owned(),
            canonical_hash: hash.to_owned(),
            source_document: "a.md".to_owned(),
        };
    }
}
