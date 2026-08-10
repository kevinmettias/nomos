//! P1: database to export to import to export yields a byte-identical bundle.
//!
//! The property is worthless over an empty store — two empty bundles are byte-identical
//! and prove nothing. So the fixture fills every table the store knows about, and
//! [`Test_Every_Table_Should_Be_Exercised`] fails if one of them is ever empty.

use nomos_spec_bundle::{Bundle, BundleError, Export, Import, Record};
use nomos_spec_model::{
    DecisionGap, FieldValue, Origin, Segment, Severity, Submission, SubmissionKind,
    SubmissionState,
};
use nomos_spec_store::{Accept_Submission, SpecificationStore, Table};

const FIRST: &str = "---\nid: V2\n---\n# Core Architecture\n\nIdentity is not a path.\n\n\
                     ## Domain Model\n\n| Model | Owns |\n| --- | --- |\n| WorkspaceContext | \
                     the workspace |\n\n```rust\nlet quoted = \"a \\\"nested\\\" string\";\n```\n";

const SECOND: &str = "---\nid: V3\n---\n# Conformance\n\nUnknown is not pass — ni\u{00f1}o, \
                      \u{4e2d}\u{6587}, \u{1f600}.\n";

const BINARY: &[u8] = &[0xFF, 0xFE, 0x00, 0x01, 0x80];

fn Populated() -> SpecificationStore
{
    return Populated_In_Reverse(false);
}

/// The same corpus, optionally written in a different order.
///
/// Insertion order is the only thing that decides `uid`, so two stores that differ only
/// in it are the test for whether anything in the bundle is ordered by a surrogate.
fn Populated_In_Reverse(reversed: bool) -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");

    let mut documents = vec![
        ("volumes/03-conformance.md", SECOND),
        ("volumes/02-core.md", FIRST),
    ];
    if reversed
    {
        documents.reverse();
    }
    else
    {
        // Not valid UTF-8, so the exporter must fall back to base64. Without it the
        // fallback arm is dead code that no round trip ever visits.
        store.Put_Blob(BINARY).expect("stores a binary blob");
    }

    for (path, text) in documents
    {
        let document = store
            .Put_Source_Document(path, "v14.36", text)
            .expect("stores the document");
        store
            .Put_Source_Blocks(document, &Segment(text))
            .expect("stores the blocks");
    }

    if reversed
    {
        store.Put_Blob(BINARY).expect("stores a binary blob");
    }

    Populate_Graph(&store);
    Populate_Submissions(&mut store);

    return store;
}

/// A submission carrying the three things `OD-SPEC-013`'s tables exist to hold: an attributed
/// value sequence, a superseded value that is still in storage, and a gap as a row.
///
/// Written through `Accept_Submission` rather than by SQL, deliberately. The round trip is
/// meant to prove that what the one write door produces survives export and import; a fixture
/// that inserted its own rows would prove the bundle round-trips rows the door might never
/// write.
fn Populate_Submissions(store: &mut SpecificationStore)
{
    let value = |field: &str, value: &str, origin: Origin| {
        return FieldValue {
            field: field.to_owned(),
            value: value.to_owned(),
            origin,
        };
    };

    let submission = Submission {
        id: "FR-ROUND-TRIP".to_owned(),
        kind: SubmissionKind::FeatureRequest,
        form_contract_version: 1,
        state: SubmissionState::Draft,
        submitted_by: "the fixture".to_owned(),
        submitted_through: "test".to_owned(),
        values: vec![
            value("title", "A submission survives a round trip", Origin::Submitted),
            value("goal", "what was first asked", Origin::Submitted),
            // Supersedes the line above for reading, and never replaces it in storage.
            value("goal", "what it became on being asked", Origin::Clarified),
            value("behaviour", "it exports and imports unchanged", Origin::Submitted),
            value("acceptance", "the bundle is byte-identical", Origin::Submitted),
            value("invariants", "none", Origin::Submitted),
        ],
        gaps: vec![DecisionGap {
            question: "whether a gap travels once closed".to_owned(),
            blocks: vec!["behaviour".to_owned()],
            severity: Severity::NonBlocking,
            closed_by: None,
        }],
    };

    Accept_Submission(store, &submission).expect("the door accepts it");
}

/// Everything above the source documents: suites, nodes, relations, statements, lineage.
///
/// Split out of the fixture rather than inlined so the document-ordering half stays
/// readable. Both halves are one fixture and neither is useful alone.
fn Populate_Graph(store: &SpecificationStore)
{
    store
        .Connection()
        .execute_batch(
            "INSERT INTO source_headings (document_uid, ordinal, depth, title)
             SELECT uid, 1, 1, 'Core Architecture' FROM source_documents
             WHERE path = 'volumes/02-core.md';
             INSERT INTO source_headings (document_uid, ordinal, depth, title)
             SELECT uid, 2, 2, 'Domain Model' FROM source_documents
             WHERE path = 'volumes/02-core.md';
             INSERT INTO source_headings (document_uid, ordinal, depth, title)
             SELECT uid, 1, 1, 'Conformance' FROM source_documents
             WHERE path = 'volumes/03-conformance.md';

             -- Two suites, one root and one not, so the round trip carries a value in both
             -- states rather than proving the column survives only when it is true.
             INSERT INTO suites (suite_id, title, authority_root)
             VALUES ('nomos', 'The Nomos specification', 1),
                    ('xvpe-seed', 'XVPE spec seed', 0);

             -- The third node is left in no suite on purpose: unrecorded is a state the
             -- bundle has to carry, and it is not the root.
             INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                                suite_uid)
             SELECT 'AGT-EXEC-001', 'requirement', 'canonical', 'record', 'Agent execution',
                    NULL, uid FROM suites WHERE suite_id = 'nomos';
             INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                                suite_uid)
             SELECT 'CON-WORKSPACE-001', 'concept', 'canonical', 'record', 'WorkspaceContext',
                    NULL, uid FROM suites WHERE suite_id = 'xvpe-seed';
             INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
                                suite_uid)
             VALUES ('REQ-RETIRED-009', 'requirement', 'superseded', 'record', 'Retired',
                     '2026-01-14T00:00:00Z', NULL);

             INSERT INTO node_aliases (alias, node_uid)
             SELECT 'AGT-010', uid FROM nodes WHERE node_id = 'AGT-EXEC-001';

             INSERT INTO node_history
             (node_uid, ordinal, event, reason, previous_event_hash, event_hash, recorded_at)
             SELECT uid, 1, 'created', 'ingested from v14.36', NULL, 'sha256:01',
                    '2026-01-01T00:00:00Z' FROM nodes WHERE node_id = 'AGT-EXEC-001';
             INSERT INTO node_history
             (node_uid, ordinal, event, reason, previous_event_hash, event_hash, recorded_at)
             SELECT uid, 2, 'content_changed', 'wording clarified by D-129', 'sha256:01',
                    'sha256:02', '2026-02-01T00:00:00Z' FROM nodes WHERE node_id = 'AGT-EXEC-001';

             INSERT INTO relation_types (name, tier, inverse_of)
             VALUES ('verifies', 'core', 'verified_by'), ('verified_by', 'core', 'verifies');

             INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
             SELECT f.uid, 'verifies', t.uid FROM nodes f, nodes t
             WHERE f.node_id = 'CON-WORKSPACE-001' AND t.node_id = 'AGT-EXEC-001';

             INSERT INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
             SELECT uid, 'AGT-EXEC-001', 'Requirement', 'Nomos shall record ancestry.',
                    'sha256:aa', 'sha256:99' FROM nodes WHERE node_id = 'AGT-EXEC-001';

             INSERT INTO lineage
             (source_block_uid, source_heading_uid, disposition, target_node_uid, target_statement)
             SELECT b.uid, NULL, 'preserved-verbatim', NULL, s.uid
             FROM source_blocks b, normative_statements s, source_documents d
             WHERE d.path = 'volumes/02-core.md' AND b.document_uid = d.uid AND b.ordinal = 2
               AND s.statement_id = 'AGT-EXEC-001';
             INSERT INTO lineage
             (source_block_uid, source_heading_uid, disposition, target_node_uid, target_statement)
             SELECT NULL, h.uid, 'preserved-normalized', n.uid, NULL
             FROM source_headings h, nodes n, source_documents d
             WHERE d.path = 'volumes/02-core.md' AND h.document_uid = d.uid AND h.ordinal = 1
               AND n.node_id = 'CON-WORKSPACE-001';
             INSERT INTO lineage
             (source_block_uid, source_heading_uid, disposition, target_node_uid, target_statement)
             SELECT b.uid, NULL, 'superseded', NULL, NULL
             FROM source_blocks b, source_documents d
             WHERE d.path = 'volumes/03-conformance.md' AND b.document_uid = d.uid
               AND b.ordinal = 1;

             -- The concept comes from one row of the domain-model table, not from the
             -- table. Without this the new column is never exercised by the round trip.
             INSERT INTO lineage
             (source_table_row_uid, disposition, target_node_uid)
             SELECT r.uid, 'preserved-verbatim', n.uid
             FROM source_table_rows r, nodes n
             WHERE r.cells_json LIKE '%WorkspaceContext%'
               AND n.node_id = 'CON-WORKSPACE-001';

             INSERT INTO omissions
             (source_block_uid, source_heading_uid, reason, justification, decision_record)
             SELECT b.uid, NULL, 'superseded', 'replaced by the v15 records', 'D-129'
             FROM source_blocks b, source_documents d
             WHERE d.path = 'volumes/03-conformance.md' AND b.document_uid = d.uid
               AND b.ordinal = 2;

             -- The front matter one document declared. It is what lets a store rebuilt from
             -- this bundle render the record back out as markdown, so a bundle that dropped
             -- it would rebuild a store that can preserve every record and author none.
             INSERT INTO record_front_matter
             (document_uid, node_uid, status, version, tags_json)
             SELECT d.uid, n.uid, 'accepted', 2, '[\"architecture\",\"identity\"]'
             FROM source_documents d, nodes n
             WHERE d.path = 'volumes/02-core.md' AND n.node_id = 'AGT-EXEC-001';

             -- Two of them, in declared order, because the order is in the file and the
             -- ordinal is the only thing that carries it.
             INSERT INTO record_relations (document_uid, ordinal, target, relation)
             SELECT d.uid, 1, 'CON-WORKSPACE-001', 'affects'
             FROM source_documents d WHERE d.path = 'volumes/02-core.md';

             INSERT INTO record_relations (document_uid, ordinal, target, relation)
             SELECT d.uid, 2, 'REQ-RETIRED-009', 'verified_by'
             FROM source_documents d WHERE d.path = 'volumes/02-core.md';",
        )
        .expect("populates every table");
}

/// The guard against a vacuous round trip.
#[test]
fn Test_Every_Table_Should_Be_Exercised()
{
    let store = Populated();

    let empty: Vec<&str> = Table::All()
        .iter()
        .filter(|table| store.Count(**table).unwrap_or(0) == 0)
        .map(|table| table.Name())
        .collect();

    assert!(
        empty.is_empty(),
        "these tables hold nothing, so the round trip says nothing about them: {empty:?}"
    );
}

/// P1.
#[test]
fn Test_A_Bundle_Should_Survive_A_Database_Round_Trip_Byte_For_Byte()
{
    let source = Populated();
    let first = Export(&source).expect("exports").Write().expect("writes");

    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");
    let report = Import(&mut rebuilt, &Bundle::Parse(&first).expect("parses")).expect("imports");

    let second = Export(&rebuilt).expect("re-exports").Write().expect("writes");

    assert_eq!(first, second, "the bundle is not a fixpoint of the round trip");
    assert!(report.records > 20, "only {} record(s) round-tripped", report.records);
}

/// P1 does not catch ordering by `uid`, because exporting in surrogate order and then
/// importing in that same order is a fixpoint too. This is the test that does: the same
/// corpus written in a different order must produce the same bundle.
#[test]
fn Test_Two_Stores_Of_The_Same_Corpus_Should_Export_Identically()
{
    let forward = Populated_In_Reverse(false);
    let backward = Populated_In_Reverse(true);

    assert_ne!(
        Surrogates(&forward),
        Surrogates(&backward),
        "both stores assigned the same surrogates, so this test proved nothing"
    );

    let first = Export(&forward).expect("exports").Write().expect("writes");
    let second = Export(&backward).expect("exports").Write().expect("writes");

    assert_eq!(first, second, "the bundle is ordered by something that is not identity");
    assert!(
        !first.contains("\"uid\""),
        "a join surrogate reached the portable authority"
    );
}

fn Surrogates(store: &SpecificationStore) -> Vec<i64>
{
    return store
        .Connection()
        .prepare("SELECT uid FROM source_documents ORDER BY path")
        .and_then(|mut statement| {
            return statement
                .query_map([], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("reads uids");
}

/// Every row counted, both directions.
#[test]
fn Test_Import_Should_Land_Every_Row_The_Bundle_Declared()
{
    let source = Populated();
    let bundle = Export(&source).expect("exports");

    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");
    Import(&mut rebuilt, &bundle).expect("imports");

    for table in Table::All()
    {
        assert_eq!(
            source.Count(*table).expect("counts"),
            rebuilt.Count(*table).expect("counts"),
            "{} did not survive the round trip",
            table.Name()
        );
    }
}

/// A blob that is not valid UTF-8 must come back byte-exact.
#[test]
fn Test_Binary_Blobs_Should_Survive_As_Bytes()
{
    let source = Populated();
    let text = Export(&source).expect("exports").Write().expect("writes");
    assert!(text.contains("\"base64\""), "the binary blob took the utf8 arm");

    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");
    Import(&mut rebuilt, &Bundle::Parse(&text).expect("parses")).expect("imports");

    let restored: Vec<u8> = rebuilt
        .Connection()
        .query_row("SELECT content FROM blobs WHERE byte_length = 5", [], |row| {
            row.get(0)
        })
        .expect("reads the blob");

    assert_eq!(restored, BINARY);
}

/// Import places a bundle beside what a store holds and never merges into it. A store
/// already holding this bundle's own content is the case that would have to merge, so it
/// is refused — and refused naming the row, rather than reporting that some table was not
/// empty, because which content collided is the reader's next question.
#[test]
fn Test_Importing_Into_A_Store_That_Already_Holds_The_Content_Should_Be_Refused()
{
    let bundle = Export(&Populated()).expect("exports");
    let mut occupied = Populated();

    let refusal = Import(&mut occupied, &bundle).expect_err("a colliding store must be refused");

    assert!(matches!(refusal, BundleError::Occupied { .. }), "{refusal}");
}

/// A reference to something the bundle does not carry is a refusal, never a NULL. A
/// silent NULL is how a lineage row stops pointing at anything while still counting as
/// a row.
#[test]
fn Test_An_Unresolvable_Reference_Should_Be_Refused()
{
    let complete = Export(&Populated()).expect("exports");

    let salvaged: Vec<Record> = complete
        .Records()
        .iter()
        .filter(|record| {
            return !matches!(record, Record::Node(node) if node.node_id == "CON-WORKSPACE-001");
        })
        .cloned()
        .collect();
    assert!(
        salvaged.len() < complete.Records().len(),
        "the negative control removed nothing"
    );

    // The exporter's own version, not a literal: this test is about a dangling reference,
    // and pinning the number here makes every migration fail it for the wrong reason.
    let broken = Bundle::New(complete.Header().schema_version, salvaged).expect("builds");
    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");

    let refusal = Import(&mut rebuilt, &broken).expect_err("a dangling reference must be refused");

    assert!(matches!(refusal, BundleError::Unresolved { .. }), "{refusal}");
    assert_eq!(
        rebuilt.Count(Table::Blobs).expect("counts"),
        0,
        "a refused import must leave nothing behind"
    );
}

/// The exporter's completeness guard, against the bug it exists for: a join that drops
/// rows. Without it the export would simply be missing a document and say so nowhere.
#[test]
fn Test_A_Row_The_Export_Query_Drops_Should_Fail_The_Export()
{
    let store = Populated();
    let connection = store.Connection();

    connection
        .pragma_update(None, "foreign_keys", "OFF")
        .expect("relaxes the constraint");
    connection
        .execute(
            "INSERT INTO source_documents (path, revision, blob_uid)
             VALUES ('volumes/99-orphan.md', 'v14.36', 999999)",
            [],
        )
        .expect("inserts an orphan");
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .expect("restores the constraint");

    let refusal = Export(&store).expect_err("a dropped row must fail the export");

    assert!(
        matches!(
            refusal,
            BundleError::Incomplete {
                ref table,
                in_store: 3,
                exported: 2
            } if table == "source_documents"
        ),
        "{refusal}"
    );
}

/// The governing records are content like any other, so they must survive the bundle.
/// If they did not, the store committed to git would be missing the records that say
/// what the store is.
#[test]
fn Test_The_Governing_Records_Should_Survive_The_Bundle()
{
    let mut seeded = SpecificationStore::In_Memory().expect("opens");
    nomos_spec_store::Seed_Governing_Records(&mut seeded).expect("seeds");

    let first = Export(&seeded).expect("exports").Write().expect("writes");

    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");
    Import(&mut rebuilt, &Bundle::Parse(&first).expect("parses")).expect("imports");

    for id in nomos_spec_store::GOVERNING_RECORD_IDS
    {
        assert!(
            rebuilt.Node_Uid(id).expect("queries").is_some(),
            "{id} did not survive the round trip"
        );
    }
    assert!(
        rebuilt.Node_Uid("ADR-DOC-001").expect("queries").is_some(),
        "the supersession target did not survive"
    );
    assert_eq!(
        Export(&rebuilt).expect("re-exports").Write().expect("writes"),
        first
    );
}

/// The fixture carries a table, and the bundle carries its rows.
///
/// `Test_Every_Table_Should_Be_Exercised` already refuses an empty `source_table_rows`, so
/// this is the half it does not cover: that the rows survive with their kinds intact.
/// Without the kind surviving, the pipe-line count and the content count collapse into
/// one number and OD-SPEC-002's whole point is lost on the far side of a round trip.
#[test]
fn Test_The_Bundle_Should_Carry_Typed_Table_Rows()
{
    let store = Populated();
    let rows = store.Count(Table::SourceTableRows).expect("counts");

    assert!(
        rows >= 3,
        "the fixture no longer holds a table, so the row round trip is not being tested"
    );

    let bundle = Export(&store).expect("exports");
    let exported = bundle
        .Records()
        .iter()
        .filter(|record| record.Table() == Table::SourceTableRows.Name())
        .count();
    assert_eq!(u32::try_from(exported).unwrap_or(u32::MAX), rows);

    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");
    Import(&mut rebuilt, &bundle).expect("imports");

    assert_eq!(rebuilt.Count(Table::SourceTableRows).expect("counts"), rows);
    assert_eq!(
        Export(&rebuilt).expect("re-exports").Write().expect("writes"),
        bundle.Write().expect("writes"),
        "the rows did not survive the round trip byte for byte"
    );

    let census = rebuilt
        .Row_Census(nomos_spec_store::RowScope::Everything)
        .expect("takes a census");
    assert_eq!(census.separator, 1, "the row kinds did not survive");
    assert_eq!(census.header, 1, "the header kind did not survive");
    assert_eq!(census.content, 1, "the data kind did not survive");
    assert_ne!(
        census.lines, census.non_separator,
        "the kinds collapsed, so both counts became one number"
    );
}

/// A concept restored from a table row must still trace to that row on the far side.
///
/// If it did not, the bundle — the authority committed to git — would carry the concept
/// and lose what produced it, which is exactly the shape the preservation ledger exists
/// to make impossible.
#[test]
fn Test_A_Lineage_To_A_Table_Row_Should_Survive_The_Round_Trip()
{
    let source = Populated();
    let bundle = Export(&source).expect("exports");

    let carried = bundle
        .Records()
        .iter()
        .filter(|record| {
            return matches!(record, Record::Lineage(lineage) if lineage.source_table_row.is_some());
        })
        .count();
    assert_eq!(carried, 1, "the fixture's row lineage did not reach the bundle");

    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");
    Import(&mut rebuilt, &bundle).expect("imports");

    let traced: String = rebuilt
        .Connection()
        .query_row(
            "SELECT r.text FROM lineage l
             JOIN source_table_rows r ON r.uid = l.source_table_row_uid
             JOIN nodes n ON n.uid = l.target_node_uid
             WHERE n.node_id = 'CON-WORKSPACE-001'",
            [],
            |row| row.get(0),
        )
        .expect("the concept traces to no row");

    assert!(traced.contains("WorkspaceContext"), "it traced to the wrong row: {traced}");
    assert_eq!(
        Export(&rebuilt).expect("re-exports").Write().expect("writes"),
        bundle.Write().expect("writes")
    );
}

/// The guard the row count cannot be: a column joins the schema and nothing exports it.
///
/// Every row still counts on both sides, so `Assert_Complete` passes and the round trip is
/// a fixpoint — the second export drops the same column the first did. Only a column-level
/// check sees it.
#[test]
fn Test_A_Column_The_Export_Does_Not_Carry_Should_Fail_The_Export()
{
    let store = Populated();
    assert!(Export(&store).is_ok(), "the fixture must export cleanly before it is broken");

    store
        .Connection()
        .execute("ALTER TABLE lineage ADD COLUMN note TEXT", [])
        .expect("adds a column");

    let refusal = Export(&store).expect_err("an uncarried column must fail the export");

    assert!(
        matches!(
            refusal,
            BundleError::UncoveredColumn { ref table, ref column }
                if table == "lineage" && column == "note"
        ),
        "{refusal}"
    );
}
