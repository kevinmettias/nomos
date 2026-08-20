//! The two spec domains, and the corpus they are both measured over.
//!
//! These two are the ones whose repetition has to alternate rather than repeat, because a
//! store rebuilt identically every time cannot show whether anything in it is ordered by a
//! surrogate.

use nomos_spec_project::{Build, Profile};
use nomos_spec_store::SpecificationStore;

/// A specification corpus small enough to read and rich enough to order wrongly.
///
/// Two documents, one carrying a table and a fenced block, so the store holds blobs,
/// documents, headings, blocks and table rows before anything above them is inserted.
const SPEC_CORE: &str = "# Core architecture\n\nIdentity is not a path.\n\n\
                         ## Domain model\n\n| Model | Owns |\n| --- | --- |\n\
                         | WorkspaceContext | the workspace |\n| BuildVariant | one build |\n\n\
                         ```rust\nlet quoted = \"a \\\"nested\\\" string\";\n```\n";

const SPEC_CONFORMANCE: &str = "# Conformance\n\nUnknown is not pass \u{2014} ni\u{00f1}o, \
                                \u{4e2d}\u{6587}, \u{1f600}.\n";

/// The same corpus, written in one of two insertion orders.
///
/// # Why the order is a parameter
///
/// Insertion order is the only thing that decides a row's `uid`, and a `uid` is the one
/// value in either domain that is not a function of the content. So two stores of this
/// corpus written in opposite orders are the instrument for "is anything here ordered by
/// a surrogate" — which is the failure mode both declarations are actually about, and the
/// one an in-process repetition over a *single* store cannot see, because repeating a pure
/// function over identical input agrees with itself whatever it is ordered by.
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

fn Spec_Store(order: Order) -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("an in-memory store opens");

    Insert_Source_Documents(&mut store, order);
    Populate_Spec_Graph(&store, order);

    return store;
}

/// The fixture's two documents and their source blocks, in whichever order this store is
/// being written in. Insertion order is the only thing that decides a row's `uid`, which is
/// what the alternating orders exist to catch.
fn Insert_Source_Documents(store: &mut SpecificationStore, order: Order)
{
    use nomos_spec_model::Segment;

    let mut documents = vec![
        ("volumes/02-core.md", SPEC_CORE),
        ("volumes/03-conformance.md", SPEC_CONFORMANCE),
    ];
    if order == Order::Backwards
    {
        documents.reverse();
    }
    for (path, text) in documents
    {
        let document = store
            .Put_Source_Document(path, "v14.36", text)
            .expect("the fixture document stores");
        store
            .Put_Source_Blocks(document, &Segment(text))
            .expect("the fixture blocks store");
    }
}

/// Everything above the documents: suites, nodes, relations, statements, lineage.
///
/// Split out so neither half runs past the line limit, and written as SQL for the reason
/// `nomos-spec-bundle`'s own fixture is: the store's authoring API deliberately does not
/// offer a way to write an arbitrary graph, and a fixture that could only build what the
/// authoring path builds would never exercise a row the ingest path produces.
fn Populate_Spec_Graph(store: &SpecificationStore, order: Order)
{
    let nodes = Nodes_In(order);

    store
        .Connection()
        .execute_batch(SPEC_SUITES_AND_HEADINGS)
        .expect("the fixture suites and headings store");
    store
        .Connection()
        .execute_batch(&nodes)
        .expect("the fixture nodes store");

    Populate_Spec_Edges(store);
}

/// The node inserts, in whichever order this store is being written in.
fn Nodes_In(order: Order) -> String
{
    if order == Order::Forwards
    {
        return SPEC_NODES.to_owned();
    }

    let mut statements: Vec<&str> = SPEC_NODES.split(";\n").collect();
    statements.reverse();

    return statements.join(";\n");
}

/// The two suites and the headings of the two documents.
const SPEC_SUITES_AND_HEADINGS: &str =
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

/// The three nodes the graph hangs from, one statement each so the order can be reversed.
const SPEC_NODES: &str =
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
     FROM suites WHERE suite_id = 'xvpe-seed';";

/// The edges, which are where a join can drop a row and an ordering can reach a surrogate.
fn Populate_Spec_Edges(store: &SpecificationStore)
{
    store
        .Connection()
        .execute_batch(SPEC_RELATIONS)
        .expect("the fixture aliases, relation types and relations store");
    store
        .Connection()
        .execute_batch(SPEC_LINEAGE)
        .expect("the fixture statements, lineage and omissions store");
}

/// What the nodes are to each other: an alias, a relation vocabulary, and two edges.
const SPEC_RELATIONS: &str =
    "INSERT INTO node_aliases (alias, node_uid)
     SELECT 'AGT-010', uid FROM nodes WHERE node_id = 'AGT-EXEC-001';

     INSERT INTO relation_types
         (name, tier, inverse_of, domain_kinds_json, range_kinds_json, max_per_node)
     VALUES ('verifies', 'core', 'verified_by', '[\"concept\"]', '[\"requirement\"]', 8),
            ('verified_by', 'core', 'verifies', '[\"requirement\"]', '[\"concept\"]', 8),
            ('affects', 'extended', NULL, '[\"decision\"]', '[\"requirement\"]', 8);

     INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
     SELECT f.uid, 'verifies', t.uid FROM nodes f, nodes t
     WHERE f.node_id = 'CDM-WORKSPACECONTEXT' AND t.node_id = 'AGT-EXEC-001';
     INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
     SELECT f.uid, 'affects', t.uid FROM nodes f, nodes t
     WHERE f.node_id = 'D-129' AND t.node_id = 'AGT-EXEC-001';";

/// What the documents are to the nodes: statements, the lineage onto them, and one omission.
const SPEC_LINEAGE: &str =
    "INSERT INTO normative_statements
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

/// The bundle a store of the fixture corpus exports, as text.
pub(crate) fn Bundle_Bytes(order: Order) -> Vec<u8>
{
    use nomos_spec_bundle::Export;

    let store = Spec_Store(order);
    let bundle = Export(&store).expect("the fixture exports");

    assert!(
        bundle.Records().len() > 20,
        "the fixture reached the bundle as only {} record(s), so agreeing with itself \
         says almost nothing",
        bundle.Records().len()
    );

    return bundle.Write().expect("the bundle writes").into_bytes();
}

/// The two profiles the projection domain is measured over.
///
/// Authored here rather than taken from `Catalogue::Shipped`, and the reason is the golden.
/// A digest over the shipped catalogue would move the day somebody adds a profile — a
/// change to the *content* the engine projects — and the failure would name the engine's
/// encoding, which is the one thing that had not changed. What this pins is the engine and
/// two profiles that live in this file.
const PROJECTION_PROFILES: &[&str] = &[
    r#"{ "id": "determinism-markdown", "title": "Determinism, rendered",
         "format": "markdown", "output": "determinism.md",
         "sections": [{ "title": "Nodes", "content": "nodes" },
                      { "title": "Relations", "content": "relations" },
                      { "title": "Documents", "content": "documents" }] }"#,
    r#"{ "id": "determinism-json", "title": "Determinism, structured",
         "format": "json", "output": "determinism.json",
         "sections": [{ "title": "Statements", "content": "statements" },
                      { "title": "Headings", "content": "headings" },
                      { "title": "Lineage", "content": "lineage" }] }"#,
];

/// Both profiles rendered over the fixture, bodies and stamps together.
///
/// The stamp is in the production deliberately. It carries the inputs digest that
/// [`nomos_spec_project::Check`] decides freshness from, so a projection whose body was
/// stable and whose stamp was not would report every generated file as stale without a
/// single byte of output having moved — a determinism defect the body alone cannot see.
pub(crate) fn Projection_Bytes(order: Order) -> Vec<u8>
{
    let store = Spec_Store(order);
    let mut rendered = Vec::new();

    for text in PROJECTION_PROFILES
    {
        Render_One_Profile(&store, text, &mut rendered);
    }

    return rendered;
}

/// One profile parsed, built and rendered: path, body and stamp in that order, so a body
/// that held still and a stamp that did not cannot hide behind each other.
fn Render_One_Profile(store: &SpecificationStore, text: &str, rendered: &mut Vec<u8>)
{
    let profile = Profile::Parse(text).expect("the fixture profile parses");
    let output = Build(store, &profile)
        // This production is the two profiles concatenated. Dropping a failed build would
        // shorten it, and the repetition that discharges the strength claim agrees over
        // the shortened bytes exactly as readily as over the whole of them.
        .unwrap_or_else(|error| panic!("{}: {error}", profile.id));

    rendered.extend_from_slice(format!("profile\t{}\n", output.path).as_bytes());
    rendered.extend_from_slice(output.body.as_bytes());
    rendered.extend_from_slice(
        output
            .Sidecar()
            .expect("the stamp renders")
            .as_bytes(),
    );
}

/// A production that alternates the insertion order of an otherwise identical corpus.
///
/// [`Verify`] repeats a production and compares the results at the declared strength. Over
/// a production that rebuilds the *same* store every time, that comparison is a test of
/// whether the code is a pure function of its input, which it plainly is — the repetition
/// would agree even if every ordering in the exporter were `ORDER BY uid`.
///
/// Alternating makes the repetition compare two stores of one corpus instead, which is
/// what the declaration actually claims: a bundle is a function of the specification, not
/// of the order the specification arrived in. The first call is always the forward order,
/// so the digest the child process and the golden are compared against does not depend on
/// how many times the closure has been called.
///
/// [`Verify`]: nomos_integration_tests::Verify
pub(crate) fn Alternating(build: fn(Order) -> Vec<u8>) -> impl Fn() -> Vec<u8>
{
    use std::cell::Cell;

    // The returned closure has to be `Fn`, because that is what `Verify` accepts, and it has
    // to remember which order the previous call used. A `Cell` over one `bool` is the whole of
    // that state: `get` and `set` of a `Copy` value, with no borrow that could fail at runtime
    // and nothing a second caller could be holding when this one writes.
    let backwards = Cell::new(false);

    return move || {
        let order = backwards.get();
        backwards.set(!order);

        return build(if order { Order::Backwards } else { Order::Forwards });
    };
}
