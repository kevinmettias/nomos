//! The populated store every assertion here starts from, and the two ways one is rebuilt.
//!
//! The property is worthless over an empty store — two empty bundles are byte-identical
//! and prove nothing. So this fixture fills every table the store knows about, and
//! [`super::fixpoint::Test_Every_Table_Should_Be_Exercised`] fails if one of them is ever
//! empty.

use nomos_spec_bundle::{Bundle, Import_Bundle};
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

pub(crate) const BINARY: &[u8] = &[0xFF, 0xFE, 0x00, 0x01, 0x80];

pub(crate) fn Populated() -> SpecificationStore
{
    return Populated_In_Reverse(false);
}

/// The same corpus, optionally written in a different order.
///
/// Insertion order is the only thing that decides `uid`, so two stores that differ only
/// in it are the test for whether anything in the bundle is ordered by a surrogate.
pub(crate) fn Populated_In_Reverse(reversed: bool) -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("In_Memory applies the schema MIGRATIONS");
    let documents = Documents_In_Order(reversed);
    if !reversed
    {
        Put_The_Binary_Blob(&mut store);
    }
    for document in documents
    {
        Put_Document(&mut store, document);
    }
    if reversed
    {
        Put_The_Binary_Blob(&mut store);
    }

    Populate_Graph(&store);
    Populate_Submissions(&mut store);

    return store;
}

/// One fixture document: the path it is stored under, and the text it holds.
///
/// The two are named here rather than passed as adjacent strings, so a call site cannot hand
/// the text over as the path.
struct Document<'a>
{
    path: &'a str,
    text: &'a str,
}

/// The fixture's two documents, in the order this store is to be written in.
fn Documents_In_Order(reversed: bool) -> Vec<Document<'static>>
{
    let mut documents = vec![
        Document {
            path: "volumes/03-conformance.md",
            text: SECOND,
        },
        Document {
            path: "volumes/02-core.md",
            text: FIRST,
        },
    ];
    if reversed
    {
        documents.reverse();
    }

    return documents;
}

/// Not valid UTF-8, so the exporter must fall back to base64. Without it the fallback arm is
/// dead code that no round trip ever visits.
///
/// Written on the far side of the documents when the order is reversed, so the uid it lands
/// on differs between the two stores.
fn Put_The_Binary_Blob(store: &mut SpecificationStore)
{
    store.Put_Blob(BINARY).expect("stores a binary blob");
}

/// One document and the blocks it segments into, which the store holds separately.
fn Put_Document(store: &mut SpecificationStore, document: Document<'_>)
{
    let uid = store
        .Put_Source_Document(document.path, "v14.36", document.text)
        .expect("stores the document");

    store
        .Put_Source_Blocks(uid, &Segment(document.text))
        .expect("stores the blocks");
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
    let submission = A_Round_Trip_Submission();

    Accept_Submission(store, &submission).expect("the door accepts it");
}

/// The submission itself, carrying an attributed value sequence, a superseded value that is
/// still in storage, and a gap as a row.
fn A_Round_Trip_Submission() -> Submission
{
    return Submission {
        id: "FR-ROUND-TRIP".to_owned(),
        kind: SubmissionKind::FeatureRequest,
        form_contract_version: 1,
        state: SubmissionState::Draft,
        submitted_by: "the fixture".to_owned(),
        submitted_through: "test".to_owned(),
        values: vec![
            Answered_Value(Answer { field: "title", value: "A submission survives a round trip" }, Origin::Submitted),
            Answered_Value(Answer { field: "goal", value: "what was first asked" }, Origin::Submitted),
            // Supersedes the line above for reading, and never replaces it in storage.
            Answered_Value(Answer { field: "goal", value: "what it became on being asked" }, Origin::Clarified),
            Answered_Value(Answer { field: "behaviour", value: "it exports and imports unchanged" }, Origin::Submitted),
            Answered_Value(Answer { field: "acceptance", value: "the bundle is byte-identical" }, Origin::Submitted),
            Answered_Value(Answer { field: "invariants", value: "none" }, Origin::Submitted),
        ],
        gaps: vec![A_Gap()],
    };
}

/// What one field was answered with.
///
/// The field and the value are named here rather than passed as adjacent strings, so a call
/// site cannot hand the value over as the field it answers.
struct Answer<'a>
{
    field: &'a str,
    value: &'a str,
}

/// One attributed field value.
fn Answered_Value(answer: Answer<'_>, origin: Origin) -> FieldValue
{
    return FieldValue {
        field: answer.field.to_owned(),
        value: answer.value.to_owned(),
        origin,
    };
}

/// The gap the fixture carries as a row.
fn A_Gap() -> DecisionGap
{
    return DecisionGap {
        question: "whether a gap travels once closed".to_owned(),
        blocks: vec!["behaviour".to_owned()],
        severity: Severity::NonBlocking,
        closed_by: None,
    };
}

/// A store rebuilt from a bundle held in memory.
pub(crate) fn Rebuilt_From(bundle: &Bundle) -> SpecificationStore
{
    let mut rebuilt = SpecificationStore::In_Memory().expect("In_Memory applies the schema MIGRATIONS");
    Import_Bundle(&mut rebuilt, bundle).expect("the fixture bundle is disjoint from the store");

    return rebuilt;
}

/// A store rebuilt from written bundle bytes and nothing else.
pub(crate) fn Rebuilt_From_Bundle_Text(written: &str) -> SpecificationStore
{
    let bundle = Bundle::Parse(written).expect("Parse inverts Write of the fixture's own bytes");

    return Rebuilt_From(&bundle);
}

/// How many records of one table the bundle carries.
pub(crate) fn Records_In(bundle: &Bundle, table: Table) -> usize
{
    return bundle
        .Records()
        .iter()
        .filter(|record| return record.Table() == table.Name())
        .count();
}

/// Everything above the source documents: suites, nodes, relations, statements, lineage.
const GRAPH: &str =
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

     INSERT INTO relation_types
     (name, tier, inverse_of, domain_kinds_json, range_kinds_json, max_per_node)
     VALUES ('verifies', 'core', 'verified_by', '[\"concept\"]', '[\"requirement\"]', 4),
            ('verified_by', 'core', 'verifies', '[\"requirement\"]', '[\"concept\"]', 4);

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
     FROM source_documents d WHERE d.path = 'volumes/02-core.md';

     -- The corpus's own declaration of the text its layout repeats. Three rows on one
     -- normalized hash, deliberately: the first two differ only in role, and the third
     -- differs in role and multiplicity too. An exporter that coalesced two declarations of
     -- one block into one row, or an importer that merged declarations naming different
     -- roles, would drop a row here and the round trip would stop being a fixpoint.
     INSERT INTO repeated_text_declarations (normalized_hash, role, multiplicity)
     VALUES ('sha256:front-matter', 'edition-line', 10),
            ('sha256:front-matter', 'suite-title', 10),
            ('sha256:front-matter', 'volume-abstract', 3);";

/// Split out of the fixture rather than inlined so the document-ordering half stays
/// readable. Both halves are one fixture and neither is useful alone.
fn Populate_Graph(store: &SpecificationStore)
{
    store
        .Connection()
        .execute_batch(GRAPH)
        .expect("populates every table");
}
