//! The store every claim in this suite is projected from, and the four lookups more than one
//! module reaches for.
//!
//! One fixture rather than one per file. The determinism claims compare two builds of the
//! same store against each other, so a fixture restated per module would let two of them
//! drift apart and the comparison would still pass.

use nomos_spec_project::{Build, Catalogue, Profile};
use nomos_spec_store::{DocumentPath, SpecificationStore};

const CORE: &str = "# Core architecture\n\nIdentity is not a path.\n\n\
                    ## Domain model\n\n| Model | Owns |\n| --- | --- |\n\
                    | WorkspaceContext | the workspace |\n| BuildVariant | one build |\n\n\
                    ```rust\nlet quoted = \"a \\\"nested\\\" string\";\n```\n";

const CONFORMANCE: &str = "# Conformance\n\nUnknown is not pass \u{2014} ni\u{00f1}o, \
                           \u{4e2d}\u{6587}, \u{1f600}.\n";

/// The order the fixture's documents and graph rows arrive in.
///
/// Named rather than a bool. `Populate_Graph(store, true)` said nothing at the call site
/// about what `true` was true of, and the whole point of the fixture is that the two
/// orders must produce the same bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Order
{
    Forwards,
    Backwards,
}

pub(crate) fn Populated() -> SpecificationStore
{
    return Populated_In_Order(Order::Forwards);
}

pub(crate) fn Populated_In_Order(order: Order) -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory applies the schema MIGRATIONS before it returns");
    let mut documents = vec![
        ("volumes/02-core.md", CORE),
        ("volumes/03-conformance.md", CONFORMANCE),
    ];
    if order == Order::Backwards
    {
        documents.reverse();
    }

    for (path, text) in documents
    {
        Put_Document(&mut store, DocumentPath(path), text);
    }
    Populate_Graph(&store, order);

    return store;
}

/// One document and the blocks it segments into, which the store holds separately.
///
/// The path arrives as a `DocumentPath` rather than as a second `&str`, so a caller cannot hand
/// the text where the path goes and have the store take it.
fn Put_Document(store: &mut SpecificationStore, path: DocumentPath<'_>, text: &str)
{
    use nomos_spec_model::Segment;

    let document = store
        .Put_Source_Document(path, "v14.36", text)
        .expect("stores the document");

    store
        .Put_Source_Blocks(document, &Segment(text))
        .expect("stores the blocks");
}

/// The suites the fixture's two documents belong to, and the headings inside them.
const SUITES_AND_HEADINGS: &str =
    "INSERT INTO suites (suite_id, title, authority_root)
     VALUES ('nomos', 'The Nomos specification', 1),
            ('xvpe-seed', 'XVPE specification seed', 0);

     INSERT INTO source_headings (document_uid, ordinal, depth, title)
     SELECT uid, 1, 1, 'Core architecture' FROM source_documents
     WHERE path = 'volumes/02-core.md';
     INSERT INTO source_headings (document_uid, ordinal, depth, title)
     SELECT uid, 2, 2, 'Domain model' FROM source_documents
     WHERE path = 'volumes/02-core.md';
     INSERT INTO source_headings (document_uid, ordinal, depth, title)
     SELECT uid, 1, 1, 'Conformance' FROM source_documents
     WHERE path = 'volumes/03-conformance.md';";

/// Every node the fixture projects, one statement per node so the set can be reversed.
const NODES: &str =
    "INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
            suite_uid)
     SELECT 'CDM-WORKSPACECONTEXT', 'concept', 'canonical', 'record',
            'WorkspaceContext', NULL, uid FROM suites WHERE suite_id = 'nomos';
     INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
            suite_uid)
     SELECT 'AGT-EXEC-001', 'requirement', 'canonical', 'record',
            'Agent execution ancestry', NULL, uid FROM suites WHERE suite_id = 'nomos';
     INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
            suite_uid)
     SELECT 'D-129', 'decision', 'canonical', 'record',
            'The store is the identity substrate', NULL, uid
     FROM suites WHERE suite_id = 'nomos';
     INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
            suite_uid)
     SELECT 'SVC-COUNTERFACTUAL', 'service', 'canonical', 'record',
            'Counterfactual Analysis Service', NULL, uid
     FROM suites WHERE suite_id = 'nomos';
     INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
            suite_uid)
     SELECT 'RMAP-R-1', 'release', 'canonical', 'record',
            'Release 1 \u{2014} deterministic check platform', NULL, uid
     FROM suites WHERE suite_id = 'nomos';
     INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
            suite_uid)
     SELECT 'SCEN-G-2', 'scenario', 'canonical', 'record',
            'End-to-end scenario: add a strategy', NULL, uid
     FROM suites WHERE suite_id = 'xvpe-seed';
     INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
            suite_uid)
     VALUES ('REQ-RETIRED-009', 'requirement', 'superseded', 'record', 'Retired',
             '2026-01-14T00:00:00Z', NULL);";

/// What each record declared in its own front matter, for the two documents this fixture
/// holds.
///
/// # Why the fixture needs this at all
///
/// Migration 5 created `record_front_matter` for the three fields the graph deliberately
/// cannot hold -- status, version and tags -- and this fixture wrote no row into it, so the
/// whole table was unexercised here while the real store carries one row per record. A
/// fixture that cannot answer a question the store answers for every record is not a smaller
/// store, it is a differently shaped one, and a filter reading that table would have had
/// nothing to read.
///
/// # Why two rows and two different statuses
///
/// `document_uid` is the primary key, so the fixture holds exactly as many rows as it holds
/// documents: two. Two is enough, and only because they disagree. One row, or two agreeing
/// rows, would let a filter that ignored the value entirely pass just as well as one that
/// read it.
const RECORD_FRONT_MATTER: &str =
    "INSERT INTO record_front_matter (document_uid, node_uid, status, version, tags_json)
     SELECT d.uid, n.uid, 'open', 1, '[\"core\"]'
     FROM source_documents d, nodes n
     WHERE d.path = 'volumes/02-core.md' AND n.node_id = 'D-129';
     INSERT INTO record_front_matter (document_uid, node_uid, status, version, tags_json)
     SELECT d.uid, n.uid, 'accepted', 2, '[\"conformance\"]'
     FROM source_documents d, nodes n
     WHERE d.path = 'volumes/03-conformance.md' AND n.node_id = 'CDM-WORKSPACECONTEXT';";

/// The aliases, relations, statements, lineage and omissions that hang off those nodes.
const GRAPH_EDGES: &str =
    "INSERT INTO node_aliases (alias, node_uid)
     SELECT 'AGT-010', uid FROM nodes WHERE node_id = 'AGT-EXEC-001';

     INSERT INTO relation_types
         (name, tier, inverse_of, domain_kinds_json, range_kinds_json, max_per_node)
     VALUES ('verifies', 'core', 'verified_by', '[\"concept\"]', '[\"requirement\"]', 8),
            ('verified_by', 'core', 'verifies', '[\"requirement\"]', '[\"concept\"]', 8),
            ('affects', 'extended', NULL, '[\"decision\"]', '[\"service\"]', 8);

     INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
     SELECT f.uid, 'verifies', t.uid FROM nodes f, nodes t
     WHERE f.node_id = 'CDM-WORKSPACECONTEXT' AND t.node_id = 'AGT-EXEC-001';
     INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
     SELECT f.uid, 'affects', t.uid FROM nodes f, nodes t
     WHERE f.node_id = 'D-129' AND t.node_id = 'SVC-COUNTERFACTUAL';

     INSERT INTO normative_statements
     (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
     SELECT uid, 'AGT-EXEC-001', 'requirement', 'Nomos shall record ancestry.',
            'sha256:aa', 'sha256:99' FROM nodes WHERE node_id = 'AGT-EXEC-001';
     INSERT INTO normative_statements
     (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
     SELECT uid, 'D-129-01', 'principle', 'The store is the identity substrate.',
            'sha256:bb', NULL FROM nodes WHERE node_id = 'D-129';

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
       AND n.node_id = 'CDM-WORKSPACECONTEXT';
     INSERT INTO lineage (source_table_row_uid, disposition, target_node_uid)
     SELECT r.uid, 'preserved-verbatim', n.uid
     FROM source_table_rows r, nodes n
     WHERE r.cells_json LIKE '%WorkspaceContext%'
       AND n.node_id = 'CDM-WORKSPACECONTEXT';

     INSERT INTO omissions
     (source_block_uid, source_heading_uid, reason, justification, decision_record)
     SELECT b.uid, NULL, 'superseded', 'replaced by the v15 records', 'D-129'
     FROM source_blocks b, source_documents d
     WHERE d.path = 'volumes/03-conformance.md' AND b.document_uid = d.uid
       AND b.ordinal = 1;";

fn Populate_Graph(store: &SpecificationStore, order: Order)
{
    let mut statements: Vec<&str> = NODES.split(";\n").collect();
    statements.reverse();
    let reversed = statements.join(";\n");

    store
        .Connection()
        .execute_batch(SUITES_AND_HEADINGS)
        .expect("populates the suites and headings");
    store
        .Connection()
        .execute_batch(if order == Order::Backwards { &reversed } else { NODES })
        .expect("populates the nodes");

    store
        .Connection()
        .execute_batch(RECORD_FRONT_MATTER)
        .expect("populates the record front matter");

    Populate_Graph_Edges(store);
}

fn Populate_Graph_Edges(store: &SpecificationStore)
{
    store
        .Connection()
        .execute_batch(GRAPH_EDGES)
        .expect("populates the graph");
}

pub(crate) fn Shipped() -> Catalogue
{
    return Catalogue::Shipped().expect("the shipped profiles parse");
}

pub(crate) fn Profile_Named(id: &str) -> Profile
{
    return Shipped()
        .Named(id)
        // Every caller names a profile out of the shipped catalogue, so `None` means a profile
        // was renamed or dropped and a test is still asking for it. The id is the whole
        // message because this helper is reached from every module in the suite and the
        // backtrace is the only other thing that would say which one.
        .unwrap_or_else(|| panic!("{id} is not a shipped profile"))
        .clone();
}

pub(crate) fn Rendered_Profile_Body(store: &SpecificationStore, id: &str) -> String
{
    return Build(store, &For_Building(&Profile_Named(id)))
        // Callers take the body and run `contains` over it, several of them negated — and a
        // negated `contains` passes over an empty string. A refusal turned into an empty body
        // would read as "the renderer emitted no raw markup". The id says which profile.
        .unwrap_or_else(|error| panic!("{id}: {error}"))
        .body;
}

/// The node the subject-addressed profiles are pointed at in this fixture.
///
/// It holds a statement and sits at the *target* end of its only relation, which is what
/// makes it the useful one: a subject shown only the edges it starts would render an empty
/// Relations section here, and every assertion below would still pass.
const SUBJECT_IN_FIXTURE: &str = "AGT-EXEC-001";

/// A profile the catalogue-wide assertions can build.
///
/// A subject-addressed profile is a template and `Build` refuses one, so a loop over every
/// shipped profile has to say which subject it means. Resolving here rather than skipping
/// is the point: skipping would quietly drop four profiles from every assertion in this
/// file, and the assertions would go on reading as though they covered the catalogue.
pub(crate) fn For_Building(profile: &Profile) -> Profile
{
    return profile
        .For(profile.Is_Per_Subject().then_some(SUBJECT_IN_FIXTURE))
        // `For` refuses a subject a whole-store profile has nowhere to put and refuses a
        // template given none, and the line above is what decides which of the two this
        // profile is. A refusal means that decision was wrong — and the doc above says why
        // returning the profile unchanged instead would drop it from every assertion here.
        .unwrap_or_else(|error| panic!("{}: {error}", profile.id));
}
