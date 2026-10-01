//! What this module promises, exercised.

use super::*;

/// A statement's recorded canonical hash, kept distinct from its text so a call site cannot
/// hand the two over in the wrong order.
struct RecordedHash<'a>(&'a str);

fn Store() -> SpecificationStore
{
    return SpecificationStore::In_Memory()
        .expect("an in-memory store opens against no file, so this construction has no failure");
}

fn Statement_File_From_Text(text: &str, hash: RecordedHash<'_>) -> StatementFile
{
    return StatementFile {
        statements: vec![RecordedStatement {
            id: "AGT-001".to_owned(),
            kind: "Requirement".to_owned(),
            canonical_text: text.to_owned(),
            canonical_hash: hash.0.to_owned(),
            source_document: "a.md".to_owned(),
        }],
    };
}

#[test]
fn Test_A_Matching_Statement_Should_Ingest_Cleanly()
{
    let text = "Nomos shall do the thing.";
    let file = Statement_File_From_Text(text, RecordedHash(ContentHash::Of(text).As_String_Slice()));

    let report = Ingest_Statements(&mut Store(), &file)
        .expect("the fixture's statement is well formed, so ingestion reports rather than refuses");

    assert!(report.Is_Passing());
    assert_eq!(report.ingested, 1);
}

/// A recorded hash that disagrees with its own recorded text is the corpus telling
/// us something is wrong. It must be reported per identifier, never summed.
#[test]
fn Test_A_Divergent_Hash_Should_Be_Reported_By_Id()
{
    let file = Statement_File_From_Text("Nomos shall do the thing.", RecordedHash("sha256:0000"));

    let report = Ingest_Statements(&mut Store(), &file)
        .expect("a hash that disagrees with its text is reported in the report, never refused");

    assert!(!report.Is_Passing());
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
    let file = Statement_File_From_Text(text, RecordedHash(ContentHash::Of(text).As_String_Slice()));

    let report = Ingest_Statements(&mut Store(), &file)
        .expect("non-canonical text is reported in the report, so ingestion itself still succeeds");

    assert!(!report.Is_Passing(), "the hash matches but the text is not canonical");
    assert!(report.divergences.is_empty());
    assert_eq!(report.non_canonical_text, vec!["AGT-001".to_owned()]);
}

#[test]
fn Test_An_Empty_Statement_File_Should_Not_Pass()
{
    let report = Ingest_Statements(&mut Store(), &StatementFile { statements: Vec::new() })
        .expect("an empty statement file is a report of nothing ingested, not a refusal");

    assert!(!report.Is_Passing(), "ingesting nothing is not a clean ingest");
}

#[test]
fn Test_Ingest_Should_Be_Idempotent()
{
    let mut store = Store();
    let markdown = "# Title\n\nOne.\n";

    let first = Ingest_Source_Document(&mut store, "a.md", "v14.36", markdown)
        .expect("a markdown document with one heading ingests as its own source document");
    let second = Ingest_Source_Document(&mut store, "a.md", "v14.36", markdown)
        .expect("re-ingesting the same document takes the same path, so it reports once more");

    assert_eq!(first, second);
    assert_eq!(
        store
            .Count(nomos_spec_store::Table::SourceBlocks)
            .expect("the store answers a count over the tables it owns"),
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
    let file = Statement_File_From_Text(text, RecordedHash(ContentHash::Of(text).As_String_Slice()));

    Ingest_Statements(&mut store, &file)
        .expect("the fixture's statement is well formed, so ingestion reports rather than refuses");

    let summary = store
        .Node_Summary("AGT-001")
        .expect("the store answers a summary lookup over the nodes it owns")
        .expect("the ingest above committed a node named AGT-001");
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

    let report = Ingest_Catalog(&mut store, &entities)
        .expect("the fixture's catalog entity is well formed, so ingestion reports rather than refuses");

    assert_eq!(report.nodes, 1);
    assert_eq!(report.aliases, 1);
    assert_eq!(
        store
            .Count(nomos_spec_store::Table::Nodes)
            .expect("the store answers a count over the tables it owns"),
        1
    );
}
