//! What this module promises, exercised.

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

/// A statement's node must answer to the catalog's own kind vocabulary — lowercase,
/// underscore-separated — or a project section filtering on `kind: "requirement"` selects
/// nothing from a store that has 324 of them, spelled `Requirement`.
#[test]
fn Test_A_Statements_Node_Kind_Should_Match_The_Catalogs_Vocabulary()
{
    let mut store = Store();
    let text = "Nomos shall do the thing.";
    let file = Statements(text, ContentHash::Of(text).As_Str());

    Ingest_Statements(&mut store, &file).expect("ingests");

    let summary = store.Node_Summary("AGT-001").expect("reads").expect("node exists");
    assert_eq!(summary.kind, "requirement");

    let stored_kind: String = store
        .Connection()
        .query_row(
            "SELECT kind FROM normative_statements WHERE statement_id = 'AGT-001'",
            [],
            |row| row.get(0),
        )
        .expect("reads the row a project section actually filters on");
    assert_eq!(stored_kind, "requirement");
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
