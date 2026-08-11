//! Every declared [`Strategy`] in this workspace, checked against the domain that
//! declared it.
//!
//! # What was wrong before this file existed
//!
//! `nomos-contracts` defined `Strategy`, `DeterminismStrength`, `ReproducibilityScope`
//! and `TraceEquivalence`, wrote a six-row table naming which domain claims what, and
//! nothing in the workspace implemented any of it. That crate is the one every non-Rust
//! peer reimplements, so an unimplemented declaration there is not an unused type — it is
//! a published protocol commitment that no implementation has ever been held to.
//!
//! The properties were not absent, only the declarations. `nomos-lang-rust` has read the
//! same corpus twice and agreed with itself since it was written, and `nomos-workspace`
//! has produced byte-identical snapshots across a hundred ingestion orders. Both of those
//! assertions are corpus-gated and neither has ever run in CI.
//!
//! # The shape of every test here
//!
//! One test per declared domain, and each one:
//!
//! 1. hands [`Verify`] the domain's own `Strategy` and a closure that produces its bytes,
//!    which discharges coherence and repetition at the declared strength;
//! 2. asks [`Cross_Environment_Owed`] what the declared *scope* additionally requires and
//!    discharges exactly that — a second process, and where the scope reaches
//!    `CrossPlatform` or above, agreement with a digest committed to this tree.
//!
//! Step 2 is deliberately not a list of things to remember. It is a function of the
//! declaration, so raising a scope raises the obligation with no second edit, and a domain
//! that cannot meet its declared scope fails here rather than in review.

use nomos_analysis::{
    Dependency, FactReuse, FactStore, GenerationCause, MemoryFactStore,
};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, DeterminismStrength, GenerationId, ReproducibilityScope,
    SnapshotId, Strategy, SubjectId, TraceEquivalence,
};
use nomos_integration_tests::{
    Child_Variable, Cross_Environment_Owed, CrossEnvironment, Digest_In, Production, Report_Line,
    Verification, Verify,
};
use nomos_lang_rust::{rollup, SyntaxFactProduction};
use nomos_lang_rust_scan::ScanFactProduction;
use nomos_model::Content_Digest;
use nomos_spec_bundle::{BundleSerialization, Export};
use nomos_spec_model::Segment;
use nomos_spec_project::{Build, Profile, ProjectionOutput};
use nomos_spec_store::SpecificationStore;
use nomos_workspace::{
    BuildVariant, ChangeSource, SnapshotSerialization, Workspace, WorkspaceChangeSet,
};
use std::cell::Cell;

// ---------------------------------------------------------------------------------
// The fixture
// ---------------------------------------------------------------------------------

/// The source every producing domain is measured over.
///
/// Small, and chosen rather than sampled. Each file carries at least one construct that
/// has historically been a source of ordering instability in an item walk — nested
/// modules, an impl block, a trait with several methods, and a macro invocation the parser
/// cannot see through. A fixture of `pub fn a() {}` would repeat itself identically under
/// any implementation at all, correct or not.
const FIXTURE: &[(&str, &str)] = &[
    (
        "src/lib.rs",
        "pub mod inner\n\
         {\n\
             pub struct Held { pub field: u32 }\n\
             pub fn one() {}\n\
             pub fn two() {}\n\
         }\n\
         pub trait Speaks\n\
         {\n\
             fn first(&self);\n\
             fn second(&self);\n\
             fn third(&self);\n\
         }\n\
         pub const NAMED: u32 = 7;\n",
    ),
    (
        "src/held.rs",
        "use std::collections::BTreeMap;\n\
         pub struct Held;\n\
         impl Held\n\
         {\n\
             pub fn build() -> BTreeMap<String, u32> { BTreeMap::new() }\n\
             pub fn other(&self) {}\n\
         }\n\
         macro_rules! generated { () => { pub fn hidden() {} } }\n\
         generated!();\n",
    ),
    (
        "src/enumerated.rs",
        "pub enum Kind\n\
         {\n\
             First,\n\
             Second,\n\
         }\n\
         pub type Alias = Kind;\n\
         pub static COUNT: usize = 2;\n\
         pub fn last() {}\n",
    ),
];

/// A context whose components are constants, which is right here and wrong elsewhere.
///
/// `tests/integration/src/context.rs` exists because a fact key built from byte-fill
/// constants rests on nothing and can never be observed to be wrong. That argument is
/// about facts a run produces and stores. These facts are produced twice inside one test
/// and compared against each other, so what the context names is irrelevant as long as
/// both productions name the same thing — and holding it fixed is what isolates the
/// producer as the only thing that could vary.
fn Fact_Context() -> nomos_lang_rust::FactContext
{
    return nomos_lang_rust::FactContext {
        snapshot: SnapshotId::From_Digest(Content_Digest(b"nomos.determinism.snapshot")),
        variant: BuildVariantId::From_Digest(Content_Digest(b"nomos.determinism.variant")),
        configuration: ConfigurationId::From_Digest(Content_Digest(
            b"nomos.determinism.configuration",
        )),
        generation: GenerationId::INITIAL,
    };
}

fn Scan_Context() -> nomos_lang_rust_scan::FactContext
{
    let shared = Fact_Context();

    return nomos_lang_rust_scan::FactContext {
        snapshot: shared.snapshot,
        variant: shared.variant,
        configuration: shared.configuration,
        generation: shared.generation,
    };
}

fn Subject_Of(path: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(path.as_bytes()));
}

// ---------------------------------------------------------------------------------
// What each domain produces
// ---------------------------------------------------------------------------------

/// The parser's facts over the fixture, rendered.
fn Parsed_Production() -> Vec<u8>
{
    let context = Fact_Context();
    let mut rendered = Vec::new();

    for (path, source) in FIXTURE
    {
        match nomos_lang_rust::Materialize(Subject_Of(path), source, context)
        {
            nomos_lang_rust::Materialization::Materialized(fact) =>
            {
                rendered.extend_from_slice(format!("file\t{path}\n").as_bytes());
                rendered.extend_from_slice(
                    format!("key\t{}\n", fact.Key().Digest()).as_bytes(),
                );
                rendered.extend_from_slice(&fact.payload.bytes);
            }
            nomos_lang_rust::Materialization::Unparseable(failure) =>
            {
                panic!("the fixture must parse; {path} did not: {failure}");
            }
        }
    }

    return rendered;
}

/// The rollup's fact over the same fixture, with the edges it declared.
///
/// The second producer in `nomos-lang-rust`, and the one whose reproducibility argument is
/// not the parser's — see [`SyntaxFactProduction`]'s doc. It is measured as its own
/// production rather than folded into [`Parsed_Production`] because a declaration is
/// discharged by what the harness runs, and a rollup summed into another domain's bytes
/// would be covered by that domain's digest without ever being the thing under test.
///
/// The edges are rendered, not only the payload. They decide what a later change
/// invalidates, so an edge list that varied between runs would leave invalidation itself
/// non-reproducible while every payload digest still agreed — which no assertion over the
/// bytes alone could see.
///
/// The fixture is walked in reverse to build the module, deliberately. The provider claims
/// the member order is its own rather than its caller's, and handing it the corpus order
/// both times would agree under an implementation that simply kept whatever it was given.
fn Rolled_Production() -> Vec<u8>
{
    let context = Fact_Context();
    let mut store = MemoryFactStore::New();

    let mut registry = nomos_capability::Registry::New();
    registry
        .Declare(nomos_cap_syntax::Capability_Contract())
        .expect("the syntax contract is declared once");
    registry
        .Offer(nomos_lang_rust::Provider_Offer())
        .expect("the parser's offer is within the ceiling");

    for (path, source) in FIXTURE
    {
        let nomos_lang_rust::Materialization::Materialized(fact) =
            nomos_lang_rust::Materialize(Subject_Of(path), source, context)
        else
        {
            panic!("the fixture must parse");
        };

        store
            .Materialize(*fact, &[] as &[Dependency])
            .expect("a fresh store accepts a first materialization");
    }

    let module = rollup::Module {
        subject: Subject_Of("the-fixture-module"),
        members: FIXTURE
            .iter()
            .rev()
            .map(|(path, source)| return rollup::ModuleMember::Of(Subject_Of(path), source))
            .collect(),
    };

    let need = nomos_capability::Requirement::New(
        nomos_cap_syntax::Capability(),
        nomos_cap_syntax::CONTRACT_VERSION,
        nomos_lang_rust::Declared_Guarantee(),
    );

    let rolled = rollup::Materialize_Index(&mut store, &rollup::Against { registry: &registry, need: &need, context }, &module)
        .expect("the rollup is not written behind the generation it names");

    // The same guard `Bundle_Bytes` carries, for the same reason. A rollup that read none
    // of its members produces a payload and a digest, and agreeing with itself across two
    // processes would then be a claim about three lines of header. The fixture has three
    // members and they all parse, so anything less means the reads missed — most likely a
    // requirement that resolved a provider whose key nobody wrote.
    assert_eq!(
        rolled.index.Answered(),
        FIXTURE.len(),
        "the rollup read {} of {} members, so this production is mostly not a rollup: {:#?}",
        rolled.index.Answered(),
        FIXTURE.len(),
        rolled.index.members
    );
    assert!(
        rolled.index.items.len() > 10,
        "the fixture reached the index as only {} item(s), so agreeing with itself says \
         almost nothing",
        rolled.index.items.len()
    );

    let mut rendered = Vec::new();
    rendered.extend_from_slice(format!("key\t{}\n", rolled.key.Digest()).as_bytes());
    rendered.extend_from_slice(&rollup::Encode_Index(&rolled.index));

    for dependency in &rolled.dependencies
    {
        rendered.extend_from_slice(
            format!("edge\t{}\t{:?}\n", dependency.key.Digest(), dependency.outcome).as_bytes(),
        );
    }

    return rendered;
}

/// The scanner's facts over the same fixture, rendered the same way.
fn Scanned_Production() -> Vec<u8>
{
    let context = Scan_Context();
    let mut rendered = Vec::new();

    for (path, source) in FIXTURE
    {
        let fact = nomos_lang_rust_scan::Materialize(Subject_Of(path), source, context);
        rendered.extend_from_slice(format!("file\t{path}\n").as_bytes());
        rendered.extend_from_slice(format!("key\t{}\n", fact.Key().Digest()).as_bytes());
        rendered.extend_from_slice(&fact.payload.bytes);
    }

    return rendered;
}

/// The fact cache after a load, a read of every key, and an invalidation.
///
/// The reuse domain's observable is not what a provider computed — that is the row above —
/// but what the store answers afterwards and what it says it threw away. So the rendering
/// is the read outcome per key and then the invalidation report, which is the artefact a
/// caller diffs when it wants to know why a build recomputed what it did.
fn Reuse_Production() -> Vec<u8>
{
    let context = Fact_Context();
    let mut store = MemoryFactStore::New();
    let mut keys = Vec::new();

    for (path, source) in FIXTURE
    {
        let nomos_lang_rust::Materialization::Materialized(fact) =
            nomos_lang_rust::Materialize(Subject_Of(path), source, context)
        else
        {
            panic!("the fixture must parse");
        };

        keys.push(fact.identity.clone());
        store
            .Materialize(*fact, &[] as &[Dependency])
            .expect("a fresh store accepts a first materialization");
    }

    let mut rendered = Vec::new();
    rendered.extend_from_slice(format!("live\t{}\n", store.Live()).as_bytes());
    rendered
        .extend_from_slice(format!("materializations\t{}\n", store.Materializations()).as_bytes());

    for identity in &keys
    {
        let outcome = match store.Current(identity, context.generation)
        {
            Some(fact) => format!("hit\t{}\t{}", fact.Key().Digest(), fact.payload.Digest()),
            None => format!("miss\t{}", identity.Key().Digest()),
        };
        rendered.extend_from_slice(outcome.as_bytes());
        rendered.push(b'\n');
    }

    let report = store.Invalidate(
        &GenerationCause::ConfigurationChanged {
            configuration: ConfigurationId::From_Digest(Content_Digest(
                b"nomos.determinism.other",
            )),
        },
        GenerationId::INITIAL.Next(),
    );

    rendered.extend_from_slice(format!("report\t{}\n", report.Report()).as_bytes());
    for key in &report.direct
    {
        rendered.extend_from_slice(format!("direct\t{}\n", key.Digest()).as_bytes());
    }

    return rendered;
}

/// The workspace snapshot's canonical bytes, after the fixture arrives through the door.
fn Snapshot_Production() -> Vec<u8>
{
    let variant = BuildVariant::New(
        "x86_64-unknown-none",
        "determinism",
        "fixed",
        ["one", "two"],
    );
    let configuration =
        ConfigurationId::From_Digest(Content_Digest(b"nomos.determinism.configuration"));
    let mut workspace = Workspace::Empty(variant, configuration);

    let mut changes = WorkspaceChangeSet::From(ChangeSource::GitCheckout);
    for (path, source) in FIXTURE
    {
        changes = changes.Present(*path, *source);
    }

    workspace
        .Apply(&changes)
        .expect("the fixture is a valid change set");

    return workspace.Snapshot().Encode();
}

// ---------------------------------------------------------------------------------
// The two spec domains
// ---------------------------------------------------------------------------------

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
enum Order
{
    Forwards,
    Backwards,
}

fn Spec_Store(order: Order) -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("an in-memory store opens");
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

    Populate_Spec_Graph(&store, order);

    return store;
}

/// Everything above the documents: suites, nodes, relations, statements, lineage.
///
/// Split out so neither half runs past the line limit, and written as SQL for the reason
/// `nomos-spec-bundle`'s own fixture is: the store's authoring API deliberately does not
/// offer a way to write an arbitrary graph, and a fixture that could only build what the
/// authoring path builds would never exercise a row the ingest path produces.
fn Populate_Spec_Graph(store: &SpecificationStore, order: Order)
{
    let nodes = "INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at,
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

    let mut statements: Vec<&str> = nodes.split(";\n").collect();
    statements.reverse();
    let reversed_nodes = statements.join(";\n");

    store
        .Connection()
        .execute_batch(
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
             WHERE path = 'volumes/03-conformance.md';",
        )
        .expect("the fixture suites and headings store");

    store
        .Connection()
        .execute_batch(if order == Order::Backwards { &reversed_nodes } else { nodes })
        .expect("the fixture nodes store");

    Populate_Spec_Edges(store);
}

/// The edges, which are where a join can drop a row and an ordering can reach a surrogate.
fn Populate_Spec_Edges(store: &SpecificationStore)
{
    store
        .Connection()
        .execute_batch(
            "INSERT INTO node_aliases (alias, node_uid)
             SELECT 'AGT-010', uid FROM nodes WHERE node_id = 'AGT-EXEC-001';

             INSERT INTO relation_types (name, tier, inverse_of)
             VALUES ('verifies', 'core', 'verified_by'), ('verified_by', 'core', 'verifies'),
                    ('affects', 'extended', NULL);

             INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
             SELECT f.uid, 'verifies', t.uid FROM nodes f, nodes t
             WHERE f.node_id = 'CDM-WORKSPACECONTEXT' AND t.node_id = 'AGT-EXEC-001';
             INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
             SELECT f.uid, 'affects', t.uid FROM nodes f, nodes t
             WHERE f.node_id = 'D-129' AND t.node_id = 'AGT-EXEC-001';

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
               AND b.ordinal = 1;",
        )
        .expect("the fixture edges store");
}

/// The bundle a store of the fixture corpus exports, as text.
fn Bundle_Bytes(order: Order) -> Vec<u8>
{
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
fn Projection_Bytes(order: Order) -> Vec<u8>
{
    let store = Spec_Store(order);
    let mut rendered = Vec::new();

    for text in PROJECTION_PROFILES
    {
        let profile = Profile::Parse(text).expect("the fixture profile parses");
        let output = Build(&store, &profile)
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

    return rendered;
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
fn Alternating(build: fn(Order) -> Vec<u8>) -> impl Fn() -> Vec<u8>
{
    let backwards = Cell::new(false);

    return move || {
        let order = backwards.get();
        backwards.set(!order);

        return build(if order { Order::Backwards } else { Order::Forwards });
    };
}

// ---------------------------------------------------------------------------------
// Goldens
// ---------------------------------------------------------------------------------

/// Digests captured on one platform, for the scopes whose claim spans platforms.
///
/// A `CrossPlatform` claim is verified by capturing reference output on one platform and
/// comparing from the others. In a repository the other platforms are not present to ask,
/// so the capture is committed and every platform's CI compares against it — which is the
/// same instrument, with the reference stored rather than fetched.
///
/// A legitimate change to an encoding changes these, and that is the point: the diff then
/// says "every fact ever keyed under the old bytes has been re-addressed", which is a
/// sentence somebody should have to read before merging.
/// Re-addressed once, deliberately, by `P10-SYNTAX-V2`: `nomos.syntax.items.v2` carries two
/// more fields per item, so every fact keyed under the v1 bytes has a new payload digest.
/// `OD-SYNTAX-002` is the sentence that had to be read before this diff merged.
const PARSED_GOLDEN: &str = "6ec4ad9fe9c81ab2358e3c3756a3f1f5";

/// The rollup's golden.
///
/// It pins more than a payload encoding. The bytes carry the derived fact's key, the index,
/// and every dependency edge with its outcome — so this constant moves if the member
/// ordering changes, if the edge order changes, or if what counts as a read changes. Each of
/// those is a change to what a later invalidation will reach, which is worth a sentence in a
/// diff.
const ROLLED_GOLDEN: &str = "a9dc834595e753e498f3a981020b2214";
const SCANNED_GOLDEN: &str = "e692ad97796279579ca5cd77764e08e5";
const SNAPSHOT_GOLDEN: &str = "1fb5fb67d666b0bb983f3b71e7e09f93";

/// The bundle's golden, and the one whose scope claim reaches furthest.
///
/// `CrossBinary` is verified by capturing a reference from one build and comparing from
/// others. The reference is this constant, and what it is worth is bounded by that: it is
/// compared by every later build of this workspace, which is a real comparison across
/// recompilation and is not yet a comparison across compiler versions.
/// `docs/records/OD-DETERMINISM-002` says so rather than implying more.
///
/// It also pins the store's schema version, which travels in the bundle header. A
/// migration therefore moves this constant, and that is right rather than unfortunate:
/// a bundle written under one schema and read under another is exactly the interchange
/// case the `CrossBinary` claim is about, and the diff is where somebody says so.
/// Moved by `P10-SUBMISSION-LAYOUT` (`86971ce`), which is the case the paragraph above
/// describes rather than an exception to it.
///
/// That commit added the `submissions`, `submission_values` and `submission_gaps` tables as
/// store schema migration 6 and extended the exporter to cover them, so both halves of what
/// this pins moved: the schema version travelling in the bundle header, and the column
/// coverage the export asserts. The first failure was not a digest mismatch at all — it was
/// `UncoveredColumn { table: "submissions", column: "uid" }`, the exporter refusing to write
/// a bundle it could not fully describe, which is that check working.
///
/// Recaptured here rather than by that commit's author because they could not: the item's
/// territory is the four `crates/spec` crates and its predicate does not reach
/// `nomos-integration-tests`, so nothing they ran could see this constant and nothing they
/// were entitled to write could change it. `OD-LEDGER-003` decided that a per-item predicate
/// stays narrower than the gate and that the gate at push time is what closes the gap, so
/// this is the designed outcome rather than a defect somebody let through — and this comment
/// is the sentence that decision expects somebody to read.
const BUNDLE_GOLDEN: &str = "0d43f057b74822e3ab0d4305d3187f14";
const PROJECTION_GOLDEN: &str = "9cecf39961bbd638111f82382eafd643";

// ---------------------------------------------------------------------------------
// The declared domains
// ---------------------------------------------------------------------------------

/// Discharges the cross-environment half of a scope claim.
///
/// Spawns this same test binary, running this same test, with [`Child_Variable`] set. The
/// child prints its digest and returns; the parent compares. That is a second process on
/// the same machine, which is exactly what `CrossRun` names — and what an in-process
/// repetition cannot reach, because hash seeds, address layout and environment are fixed
/// for the life of a process and vary between two.
///
/// # Why the child's early return is not the defect this workspace hunts
///
/// A test that returns early prints `ok`, and `docs/records/OD-GATE-001` is about exactly
/// that. The branch below is not an instance of it: it is only ever taken in a process the
/// parent spawned, and the parent asserts on what it printed. No run of the suite reaches
/// the early return without a run of the suite having made the assertion. A `cargo test`
/// on a machine with nothing configured takes the full path.
fn Discharge_Scope<S: Strategy>(domain: &str, verification: &Verification, golden: &str)
{
    match Cross_Environment_Owed(S::SCOPE)
    {
        CrossEnvironment::Nothing =>
        {}
        CrossEnvironment::SecondProcess =>
        {
            Compare_Against_Child(domain, verification);
        }
        CrossEnvironment::SecondProcessAndGolden =>
        {
            Compare_Against_Child(domain, verification);
            assert_eq!(
                verification.digest, golden,
                "{domain} declares {} and its bytes do not match the digest captured for \
                 another platform. Either this platform produces different bytes — in which \
                 case the declaration is false and the implementation or the scope must \
                 change — or the encoding changed deliberately, in which case every fact \
                 ever keyed under the old bytes has just been re-addressed and this constant \
                 is the place that says so.",
                S::SCOPE
            );
        }
    }
}

fn Compare_Against_Child(domain: &str, verification: &Verification)
{
    let executable = std::env::current_exe().expect("a running test has an executable path");

    let output = std::process::Command::new(executable)
        .args(["--exact", "--nocapture", Test_Name_For(domain)])
        .env(Child_Variable(), domain)
        .output()
        .expect("the test binary must be runnable as a child; a scope claim it cannot verify is a scope claim nothing checks");

    let printed = String::from_utf8_lossy(&output.stdout).into_owned();

    assert!(
        output.status.success(),
        "the child process running {domain} failed: {printed}{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let reported = Digest_In(&printed, domain).unwrap_or_else(|| {
        panic!("the child running {domain} printed no digest line; it printed: {printed}")
    });

    assert_eq!(
        reported, verification.digest,
        "{domain} produced different bytes in a second process. That is the failure a \
         CrossRun claim exists to catch: an unordered collection, a hash seed, or an \
         address reaching the output."
    );
}

/// The test function name a domain's child process should run.
///
/// Written down rather than derived from `std::any::type_name` or a macro, because the
/// child is selected by `--exact` and a name that drifted from the function would spawn a
/// child that ran *no* tests, exited zero, printed nothing, and failed only on the missing
/// digest line — a failure that names the wrong cause.
fn Test_Name_For(domain: &str) -> &'static str
{
    return match domain
    {
        "syntax-fact-production" => "Test_The_Parser_Should_Meet_Its_Declared_Strategy",
        "module-index-rollup" => "Test_The_Rollup_Should_Meet_Its_Declared_Strategy",
        "scan-fact-production" => "Test_The_Scanner_Should_Meet_Its_Declared_Strategy",
        "fact-reuse" => "Test_The_Fact_Cache_Should_Meet_Its_Declared_Strategy",
        "snapshot-serialization" => "Test_Snapshot_Serialization_Should_Meet_Its_Declared_Strategy",
        "bundle-serialization" => "Test_Bundle_Serialization_Should_Meet_Its_Declared_Strategy",
        "projection-output" => "Test_Projection_Output_Should_Meet_Its_Declared_Strategy",
        other => panic!("no test is registered for the domain {other}"),
    };
}

/// Whether this process is a child spawned to report on one domain.
fn Child_For(domain: &str) -> bool
{
    return std::env::var(Child_Variable()).is_ok_and(|asked| return asked == domain);
}

/// The whole of one domain's obligation, discharged.
fn Check<S: Strategy>(domain: &str, produce: &dyn Fn() -> Vec<u8>, golden: &str)
{
    // A child reports and returns. It must not spawn a child of its own, which would
    // recurse until the machine ran out of processes.
    if Child_For(domain)
    {
        let production = Production { trace: produce() };
        println!(
            "{}",
            Report_Line(domain, &production.Digest_At(S::STRENGTH))
        );
        return;
    }

    let verification = Verify::<S>(domain, produce);
    Discharge_Scope::<S>(domain, &verification, golden);

    eprintln!(
        "{domain}: {} / {} / {} — discharged {}",
        S::STRENGTH,
        S::SCOPE,
        S::TRACE,
        verification.discharged.join(", ")
    );
}

#[test]
fn Test_The_Parser_Should_Meet_Its_Declared_Strategy()
{
    Check::<SyntaxFactProduction>(
        "syntax-fact-production",
        &Parsed_Production,
        PARSED_GOLDEN,
    );
}

/// The second producer covered by `nomos-lang-rust`'s declaration, discharged separately.
///
/// Same `Strategy`, because it is the same execution domain holding the same triple —
/// `SyntaxFactProduction`'s doc says why one declaration covers both. What must not be
/// shared is the *production*: a declaration is only as good as what the harness runs it
/// over, and this is the run that makes the crate's promise true of the rollup rather than
/// merely stated about it.
#[test]
fn Test_The_Rollup_Should_Meet_Its_Declared_Strategy()
{
    Check::<SyntaxFactProduction>("module-index-rollup", &Rolled_Production, ROLLED_GOLDEN);
}

#[test]
fn Test_The_Scanner_Should_Meet_Its_Declared_Strategy()
{
    Check::<ScanFactProduction>("scan-fact-production", &Scanned_Production, SCANNED_GOLDEN);
}

#[test]
fn Test_The_Fact_Cache_Should_Meet_Its_Declared_Strategy()
{
    // No golden. `FactReuse` declares `CrossRun`, and `Cross_Environment_Owed` therefore
    // never reaches for one — passing a real digest here would be a check the declaration
    // did not ask for, which is the same defect as a missing one pointed the other way.
    Check::<FactReuse>("fact-reuse", &Reuse_Production, "");
}

#[test]
fn Test_Snapshot_Serialization_Should_Meet_Its_Declared_Strategy()
{
    Check::<SnapshotSerialization>(
        "snapshot-serialization",
        &Snapshot_Production,
        SNAPSHOT_GOLDEN,
    );
}

#[test]
fn Test_Bundle_Serialization_Should_Meet_Its_Declared_Strategy()
{
    Check::<BundleSerialization>(
        "bundle-serialization",
        &Alternating(Bundle_Bytes),
        BUNDLE_GOLDEN,
    );
}

#[test]
fn Test_Projection_Output_Should_Meet_Its_Declared_Strategy()
{
    Check::<ProjectionOutput>(
        "projection-output",
        &Alternating(Projection_Bytes),
        PROJECTION_GOLDEN,
    );
}

// ---------------------------------------------------------------------------------
// The controls
// ---------------------------------------------------------------------------------

/// A declaration with the bundle row's triple, over a domain that does not hold it.
///
/// Held here rather than in a crate because it is not a domain: it is the negative control
/// for the harness, and a strategy declared in a `src/` directory would be found by
/// `Test_Every_Declaration_Should_Be_Held_To_It_By_The_Harness` and correctly reported as a
/// promise with nothing behind it.
struct Wobbly;

impl Strategy for Wobbly
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::CrossBinary;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

/// The control that says the harness would catch a real violation.
///
/// Without it every assertion above is consistent with a [`Verify`] that compares nothing:
/// a domain repeated eight times and found to agree proves the domain is a function of its
/// input, and proves nothing whatever about the instrument. So this runs the instrument
/// over a production that is *not* a function of its input and asserts it fails, with the
/// message it fails by, at the declared strength.
///
/// The variation is a counter rather than a hash seed or a clock, because a control has to
/// fail on every machine on every run — a control that is itself flaky is a control nobody
/// believes when it goes green.
#[test]
fn Test_A_Domain_That_Does_Not_Repeat_Itself_Should_Fail_The_Harness()
{
    let call = Cell::new(0_u32);
    let wobbles = || {
        let seen = call.get();
        call.set(seen.saturating_add(1));

        return format!("line\tone\nline\t{seen}\n").into_bytes();
    };

    let refusal = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        return Verify::<Wobbly>("wobbly", &wobbles);
    }))
    .expect_err("a production that changes between repetitions must fail the harness");

    let said = refusal
        .downcast_ref::<String>()
        .map_or_else(String::new, Clone::clone);
    assert!(
        said.contains("produced a different set"),
        "the harness failed for the wrong reason: {said}"
    );
    assert!(
        call.get() > 1,
        "the control never reached a second repetition, so it proved nothing"
    );
}

/// The control for the golden, which is the half of a scope claim a repetition cannot make.
///
/// A committed digest only catches a changed encoding if it is a function of the bytes.
/// Both spec domains declare `State`, and a `State` digest is taken over the *sorted* line
/// set — so the reasonable worry is that it is insensitive to something it ought to catch.
/// This shows it is not: one altered byte anywhere in a real bundle moves the digest the
/// golden is compared against.
#[test]
fn Test_An_Altered_Byte_Should_Move_The_Digest_The_Golden_Pins()
{
    let honest = Production {
        trace: Bundle_Bytes(Order::Forwards),
    };
    let mut altered = honest.trace.clone();
    let last = altered
        .iter()
        .rposition(|byte| return byte.is_ascii_lowercase())
        .expect("a bundle carries lower-case text");
    let target = altered
        .get_mut(last)
        .expect("the position just found is in range");
    *target = target.to_ascii_uppercase();

    let tampered = Production { trace: altered };

    assert_ne!(
        tampered.trace, honest.trace,
        "the control altered nothing, so the comparison below cannot fail"
    );
    assert_ne!(
        tampered.Digest_At(DeterminismStrength::State),
        honest.Digest_At(DeterminismStrength::State),
        "a changed byte left the digest where it was, so the golden pins nothing"
    );
    assert_eq!(
        honest.Digest_At(DeterminismStrength::State),
        BUNDLE_GOLDEN,
        "the honest half of this control must be the bundle the golden was captured from"
    );
}

// ---------------------------------------------------------------------------------
// The declarations themselves
// ---------------------------------------------------------------------------------

/// Every domain this workspace has, with the row of the contracts table it occupies.
///
/// # Why this test was renamed
///
/// It was `Test_The_Declared_Domains_Should_Be_The_Ones_This_Item_Covered`, and
/// `docs/records/OD-DETERMINISM-001` cites it by that name as the place the two undeclared
/// rows were written down. Both are declared now, so the sentence that name asserts is
/// false. A citation that resolves to a test asserting the opposite of what the citing
/// record says is worse than one that resolves to nothing, so the name moved and
/// `docs/records/OD-DETERMINISM-002` records where it went.
///
/// # What is still not covered, and why that is not a gap
///
/// Two of the table's six rows have no domain in this tree. "Correction planning and
/// staging" describes work that applies fixes, and nothing here applies one. "Progress UI,
/// logs, telemetry, agent execution" is the `None` row — the CLI prints, and nothing about
/// what it prints is a fact. Neither is an omission that a declaration would repair;
/// `tests/contract/tests/determinism_declarations.rs` is where they are accounted for, so
/// that a crate arriving to occupy either row cannot do so silently.
#[test]
fn Test_Every_Domain_In_The_Tree_Should_Declare_And_Be_Registered()
{
    let declared = [
        ("syntax-fact-production", SyntaxFactProduction::STRENGTH),
        // The same declaration, discharged over the other thing it covers. Two entries and
        // one strategy is the shape `P10-ROLLUP-DETERMINISM` settled on: `nomos-lang-rust`
        // has two fact producers occupying one row of the contracts table, so they are one
        // promise — and a promise covering two producers has to be run over both, or the
        // second is covered by a sentence and measured by nothing.
        ("module-index-rollup", SyntaxFactProduction::STRENGTH),
        ("scan-fact-production", ScanFactProduction::STRENGTH),
        ("fact-reuse", FactReuse::STRENGTH),
        ("snapshot-serialization", SnapshotSerialization::STRENGTH),
        ("bundle-serialization", BundleSerialization::STRENGTH),
        ("projection-output", ProjectionOutput::STRENGTH),
    ];

    assert_eq!(
        declared.len(),
        7,
        "seven productions are covered by six declarations; a new producer needs a row in \
         this table and a test of its own, whether or not it also needs a declaration of \
         its own"
    );

    for (domain, strength) in declared
    {
        assert!(
            !Test_Name_For(domain).is_empty(),
            "{domain} declares a strategy and has no test registered"
        );
        assert_ne!(
            strength,
            DeterminismStrength::None,
            "{domain} is measured here and promises nothing, so the measurement is \
             discharging no obligation"
        );
    }
}
