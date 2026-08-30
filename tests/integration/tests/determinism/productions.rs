//! The fixture, and the bytes each in-tree domain produces over it.
//!
//! One source fixture for all five, so that a difference between two productions is a
//! difference between the producers rather than between what they were shown.

use nomos_analysis::{
    Dependency, FactIdentity, FactStore, GenerationCause, MemoryFactStore,
};
use nomos_contracts::{
    BuildVariantId, ConfigurationId, EvidenceClass, GenerationId, ProviderId, SnapshotId, SubjectId,
};
use nomos_corrections::{ChangeSet, CorrectionCandidate, CorrectionClass, CorrectionPlan, Edit};
use nomos_lang_rust::rollup;
use nomos_model::{Content_Digest, Evidence};
use nomos_platform_std::StdProcessLauncher;
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};

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

/// The parser's facts over the fixture, rendered.
pub(crate) fn Parsed_Production() -> Vec<u8>
{
    let context = Fact_Context();
    let mut rendered = Vec::new();

    for (path, source) in FIXTURE
    {
        let one = Rendered_Fact(path, source, context);

        rendered.extend_from_slice(&one);
    }

    return rendered;
}

/// One fixture file's fact: the path it came from, the key it landed under, and its payload.
fn Rendered_Fact(path: &str, source: &str, context: nomos_lang_rust::FactContext) -> Vec<u8>
{
    let fact = match nomos_lang_rust::Materialize_Syntax_Fact(Subject_Of(path), source, context)
    {
        nomos_lang_rust::Materialization::Materialized(fact) => fact,
        nomos_lang_rust::Materialization::Unparseable(failure) =>
        {
            // The fixture is a constant in this file, chosen for the constructs it carries. A
            // refusal means the production below is rendered over fewer files than the
            // fixture names, and a shorter production repeats itself just as identically —
            // the domain would discharge its strength claim having read less than it says.
            panic!("the fixture must parse; {path} did not: {failure}");
        }
    };
    let mut rendered = Vec::new();

    rendered.extend_from_slice(format!("file\t{path}\n").as_bytes());
    rendered.extend_from_slice(format!("key\t{}\n", fact.Key().Digest()).as_bytes());
    rendered.extend_from_slice(&fact.payload.bytes);

    return rendered;
}

/// The reachability offer's facts over the same fixture, rendered.
///
/// This item's fixture files declare no `Err(applicability)` arm at all, so every rendered
/// fact carries an empty site list — the payload is still real bytes with a real digest,
/// and the determinism claim under test is that two runs over the same source produce the
/// identical empty answer, not that the fixture exercises the walker's positive case.
/// `crates/languages/nomos-lang-rust/src/reachability/provider.rs`'s own tests already
/// cover the positive case against synthetic sources.
pub(crate) fn Reachability_Production() -> Vec<u8>
{
    let context = Fact_Context();
    let mut rendered = Vec::new();

    for (path, source) in FIXTURE
    {
        let one = Rendered_Reachability_Fact(path, source, context);

        rendered.extend_from_slice(&one);
    }

    return rendered;
}

/// One fixture file's reachability fact: the path it came from, the key it landed under,
/// and its payload.
fn Rendered_Reachability_Fact(path: &str, source: &str, context: nomos_lang_rust::FactContext) -> Vec<u8>
{
    let fact = match nomos_lang_rust::reachability::Materialize_Reachability_Fact(Subject_Of(path), source, context)
    {
        nomos_lang_rust::Materialization::Materialized(fact) => fact,
        nomos_lang_rust::Materialization::Unparseable(failure) =>
        {
            panic!("the fixture must parse; {path} did not: {failure}");
        }
    };
    let mut rendered = Vec::new();

    rendered.extend_from_slice(format!("file\t{path}\n").as_bytes());
    rendered.extend_from_slice(format!("key\t{}\n", fact.Key().Digest()).as_bytes());
    rendered.extend_from_slice(&fact.payload.bytes);

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
///
/// [`SyntaxFactProduction`]: nomos_lang_rust::SyntaxFactProduction
pub(crate) fn Rolled_Production() -> Vec<u8>
{
    let context = Fact_Context();
    let registry = Syntax_Registry();
    let need = Syntax_Requirement();
    let module = Fixture_Module();
    let mut store = MemoryFactStore::New();

    Fill_With_Fixture(&mut store, context);

    let against = rollup::Against {
        registry: &registry,
        need: &need,
        context,
    };
    let rolled = rollup::Materialize_Index(&mut store, &against, &module)
        .expect("the rollup is not written behind the generation it names");

    Assert_The_Rollup_Read_Its_Members(&rolled);

    return Rendered_Rollup(&rolled);
}

/// The registry the rollup resolves its provider through.
fn Syntax_Registry() -> nomos_capability::Registry
{
    let mut registry = nomos_capability::Registry::New();

    registry
        .Declare(nomos_cap_syntax::Capability_Contract())
        .expect("the syntax contract is declared once");
    registry
        .Offer(nomos_lang_rust::Provider_Offer())
        .expect("the parser's offer is within the ceiling");

    return registry;
}

/// What the rollup requires of whichever provider answers it.
fn Syntax_Requirement() -> nomos_capability::Requirement
{
    return nomos_capability::Requirement::New(
        nomos_cap_syntax::Capability(),
        nomos_cap_syntax::CONTRACT_VERSION,
        nomos_lang_rust::Declared_Guarantee(),
    );
}

/// The fixture as one module, walked in reverse.
///
/// Deliberately reversed. The provider claims the member order is its own rather than its
/// caller's, and handing it the corpus order both times would agree under an implementation
/// that simply kept whatever it was given.
fn Fixture_Module() -> rollup::Module
{
    return rollup::Module {
        subject: Subject_Of("the-fixture-module"),
        members: FIXTURE
            .iter()
            .rev()
            .map(|(path, source)| return rollup::Member::Of(Subject_Of(path), source))
            .collect(),
    };
}

/// Every fixture file materialized into a store, and the identities they landed under.
fn Fill_With_Fixture(
    store: &mut MemoryFactStore,
    context: nomos_lang_rust::FactContext,
) -> Vec<FactIdentity>
{
    let mut keys = Vec::new();

    for (path, source) in FIXTURE
    {
        let nomos_lang_rust::Materialization::Materialized(fact) =
            nomos_lang_rust::Materialize_Syntax_Fact(Subject_Of(path), source, context)
        else
        {
            // The keys returned from here are what the fact-reuse domain replays against the
            // store. A file that did not materialize leaves the store short one fact and the
            // key list short the matching one, so the replay and the store would still agree
            // with each other perfectly — about a fixture neither of them holds.
            panic!("the fixture must parse");
        };

        keys.push(fact.identity.clone());
        store
            .Materialize(*fact, &[] as &[Dependency])
            .expect("a fresh store accepts a first materialization");
    }

    return keys;
}

/// The same guard `Bundle_Bytes` carries, for the same reason.
///
/// A rollup that read none of its members produces a payload and a digest, and agreeing with
/// itself across two processes would then be a claim about three lines of header. The fixture
/// has three members and they all parse, so anything less means the reads missed — most
/// likely a requirement that resolved a provider whose key nobody wrote.
fn Assert_The_Rollup_Read_Its_Members(rolled: &rollup::Rolled)
{
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
}

/// The rollup's key, its encoded index, and every edge it declared.
///
/// The edges are rendered, not only the payload. They decide what a later change
/// invalidates, so an edge list that varied between runs would leave invalidation itself
/// non-reproducible while every payload digest still agreed — which no assertion over the
/// bytes alone could see.
fn Rendered_Rollup(rolled: &rollup::Rolled) -> Vec<u8>
{
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

/// `nomos-lang-rust-cargo`'s facts over this repository's own real workspace.
///
/// Not the shared `FIXTURE` above: this provider's whole reason for existing is that it
/// reads Cargo's own resolution of a real workspace, not bytes a caller already holds.
/// This repository is that real workspace, the same real subject
/// `nomos-lang-rust-cargo`'s own crate tests and `tests/contract`'s boundary tests already
/// measure against, so nothing new is being trusted to stay stable.
///
/// Rendered in package-name order rather than in whatever order `Discover_Workspace`
/// returned. `DependencyFactProduction` declares `State`, not `StateTemporal` — the final
/// set of facts is the claim, not the order they arrived in — and sorting here is what
/// makes the test measure exactly that claim rather than a stronger one nobody declared.
pub(crate) fn Dependency_Production() -> Vec<u8>
{
    let context = Dependency_Context();
    let facts = Discovered_Workspace_Facts(context);

    return Rendered_Dependency_Facts(facts);
}

/// A context whose components are constants — see [`Fact_Context`] for why that is right
/// here and wrong elsewhere.
fn Dependency_Context() -> nomos_lang_rust_cargo::FactContext
{
    return nomos_lang_rust_cargo::FactContext {
        snapshot: SnapshotId::From_Digest(Content_Digest(b"nomos.determinism.snapshot")),
        variant: BuildVariantId::From_Digest(Content_Digest(b"nomos.determinism.variant")),
        configuration: ConfigurationId::From_Digest(Content_Digest(
            b"nomos.determinism.configuration",
        )),
        generation: GenerationId::INITIAL,
    };
}

/// This repository's own real workspace, materialized through the door this provider
/// actually reads Cargo's resolution through, and checked to have found enough of it to
/// have measured something.
fn Discovered_Workspace_Facts(
    context: nomos_lang_rust_cargo::FactContext,
) -> Vec<nomos_lang_rust_cargo::PackageFact>
{
    let facts =
        nomos_lang_rust_cargo::Materialize_Workspace(&Repository_Root(), context, &StdProcessLauncher)
            .expect("this repository is a real cargo workspace; a provider that cannot see it \
                     verifies nothing");

    assert!(
        facts.len() > 10,
        "this repository has far more than ten workspace members, so {} is too few to have \
         measured much: {facts:?}",
        facts.len()
    );

    return facts;
}

/// The facts in package-name order rather than in whatever order `Discover_Workspace`
/// returned. `DependencyFactProduction` declares `State`, not `StateTemporal` — the final
/// set of facts is the claim, not the order they arrived in — and sorting here is what makes
/// the test measure exactly that claim rather than a stronger one nobody declared.
fn Rendered_Dependency_Facts(mut facts: Vec<nomos_lang_rust_cargo::PackageFact>) -> Vec<u8>
{
    facts.sort_by(|left, right| return left.fact.payload.bytes.cmp(&right.fact.payload.bytes));

    let mut rendered = Vec::new();
    for fact in &facts
    {
        rendered.extend_from_slice(format!("key\t{}\n", fact.fact.Key().Digest()).as_bytes());
        rendered.extend_from_slice(&fact.fact.payload.bytes);
    }

    return rendered;
}

/// `nomos-lang-rust-clippy`'s facts over this repository's own real workspace.
///
/// The identical reasoning [`Dependency_Production`] gives for using the real repository
/// rather than the shared `FIXTURE`: this provider's whole reason for existing is that it
/// reads `cargo clippy`'s own real analysis, not bytes a caller already holds.
pub(crate) fn Lint_Production() -> Vec<u8>
{
    let context = Lint_Context();
    let facts = Discovered_Lint_Facts(context);

    return Rendered_Lint_Facts(facts);
}

fn Lint_Context() -> nomos_lang_rust_clippy::FactContext
{
    return nomos_lang_rust_clippy::FactContext {
        snapshot: SnapshotId::From_Digest(Content_Digest(b"nomos.determinism.snapshot")),
        variant: BuildVariantId::From_Digest(Content_Digest(b"nomos.determinism.variant")),
        configuration: ConfigurationId::From_Digest(Content_Digest(b"nomos.determinism.configuration")),
        generation: GenerationId::INITIAL,
    };
}

/// This repository's own real workspace, materialized through the door this provider
/// actually reads `cargo clippy`'s own analysis through, and checked to have found enough
/// of it to have measured something.
fn Discovered_Lint_Facts(context: nomos_lang_rust_clippy::FactContext) -> Vec<nomos_lang_rust_clippy::DiagnosticsFact>
{
    let facts = nomos_lang_rust_clippy::Materialize_Workspace(&Repository_Root(), context, &StdProcessLauncher)
        .expect("this repository is a real cargo workspace under clippy; a provider that cannot see it verifies nothing");

    assert!(
        facts.len() > 10,
        "this repository has far more than ten workspace members, so {} is too few to have \
         measured much: {facts:?}",
        facts.len()
    );

    return facts;
}

/// The facts in payload-byte order rather than in whatever order `Discover_Workspace`
/// returned — the identical reasoning [`Rendered_Dependency_Facts`] gives: `LintFactProduction`
/// declares `State`, not `StateTemporal`, so the final set of facts is the claim, not the
/// order they arrived in.
fn Rendered_Lint_Facts(mut facts: Vec<nomos_lang_rust_clippy::DiagnosticsFact>) -> Vec<u8>
{
    facts.sort_by(|left, right| return left.fact.payload.bytes.cmp(&right.fact.payload.bytes));

    let mut rendered = Vec::new();
    for fact in &facts
    {
        rendered.extend_from_slice(format!("key\t{}\n", fact.fact.Key().Digest()).as_bytes());
        rendered.extend_from_slice(&fact.fact.payload.bytes);
    }

    return rendered;
}

/// `nomos-lang-rust-deny`'s one fact over this repository's own real workspace.
///
/// The identical reasoning [`Lint_Production`] gives for using the real repository rather
/// than the shared `FIXTURE`: this provider's whole reason for existing is that it reads
/// `cargo deny`'s own real verdict, not bytes a caller already holds. No sort before
/// rendering, unlike [`Rendered_Lint_Facts`]: `IncrementalGranularity::WholeWorkspace`
/// means there is exactly one fact here, not a list whose order needs pinning down.
pub(crate) fn Dependency_Policy_Production() -> Vec<u8>
{
    let context = Dependency_Policy_Context();
    let fact = Discovered_Dependency_Policy_Fact(context);

    return Rendered_Dependency_Policy_Fact(&fact);
}

fn Dependency_Policy_Context() -> nomos_lang_rust_deny::FactContext
{
    return nomos_lang_rust_deny::FactContext {
        snapshot: SnapshotId::From_Digest(Content_Digest(b"nomos.determinism.snapshot")),
        variant: BuildVariantId::From_Digest(Content_Digest(b"nomos.determinism.variant")),
        configuration: ConfigurationId::From_Digest(Content_Digest(b"nomos.determinism.configuration")),
        generation: GenerationId::INITIAL,
    };
}

/// This repository's own real workspace, materialized through the door this provider
/// actually reads `cargo deny`'s own verdict through.
fn Discovered_Dependency_Policy_Fact(context: nomos_lang_rust_deny::FactContext) -> nomos_lang_rust_deny::PolicyFact
{
    return nomos_lang_rust_deny::Materialize_Workspace(&Repository_Root(), context, &StdProcessLauncher)
        .expect("this repository is a real cargo workspace under this repository's own deny.toml; a provider that cannot see it verifies nothing");
}

fn Rendered_Dependency_Policy_Fact(fact: &nomos_lang_rust_deny::PolicyFact) -> Vec<u8>
{
    let mut rendered = Vec::new();
    rendered.extend_from_slice(format!("key\t{}\n", fact.fact.Key().Digest()).as_bytes());
    rendered.extend_from_slice(&fact.fact.payload.bytes);

    return rendered;
}

/// `nomos-lang-go-modules`'s facts over a small, synthetic Go workspace.
///
/// Not this repository's own real tree, unlike [`Dependency_Production`] and
/// [`Lint_Production`] above: this repository is a Cargo workspace with no `go.mod`
/// anywhere in it, and `nomos-lang-go-modules`'s whole reason for existing is a shape
/// Cargo's tree does not have to offer. A small, hand-written two-module workspace,
/// written to a fresh temp directory and removed afterward, is the real subject instead —
/// real files on a real filesystem, the identical kind of subject
/// `nomos_lang_go_modules::discovery`'s own tests already read, just built here rather
/// than committed, for the same reason `GO_FIXTURE` above is a string literal rather than
/// a committed `.go` file.
pub(crate) fn Go_Dependency_Production() -> Vec<u8>
{
    let workspace = Go_Workspace_Fixture();
    let context = Go_Dependency_Context();
    let facts = nomos_lang_go_modules::Materialize_Workspace(&workspace.root, context)
        .expect("the fixture is a real two-module Go workspace");

    assert_eq!(
        facts.len(),
        2,
        "the fixture declares exactly two modules: {facts:?}"
    );

    return Rendered_Go_Dependency_Facts(facts);
}

/// A fresh temp directory holding a two-module `go.work` workspace, one module requiring
/// the other, removed when it goes out of scope.
struct GoWorkspaceFixture
{
    root: std::path::PathBuf,
}

impl GoWorkspaceFixture
{
    fn Write(&self, relative: &str, content: &str)
    {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent()
        {
            std::fs::create_dir_all(parent).expect("the fixture's own parent directory can be created");
        }
        std::fs::write(&path, content).expect("the fixture file can be written");
    }
}

impl Drop for GoWorkspaceFixture
{
    fn drop(&mut self)
    {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn Go_Workspace_Fixture() -> GoWorkspaceFixture
{
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    // Unique per call, not merely per process: `Assert_Meets_Declared_Strategy` can invoke
    // this production more than once within one process (the parent's own repeat-check and
    // its own fresh-production re-check, alongside whatever a spawned child does in its own
    // process), and a name shared across calls would let one call's `Drop` remove a
    // directory another is still reading.
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "nomos-determinism-go-dependency-{}-{n}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).expect("a fresh temp directory can be created");
    let fixture = GoWorkspaceFixture { root };

    fixture.Write("go.work", "go 1.21\n\nuse (\n\t./held\n\t./writer\n)\n");
    fixture.Write(
        "held/go.mod",
        "module example.com/held\n\ngo 1.21\n\nrequire (\n\texample.com/writer v0.0.0\n\tgithub.com/external/thing v1.0.0\n)\n",
    );
    fixture.Write("writer/go.mod", "module example.com/writer\n\ngo 1.21\n");

    return fixture;
}

fn Go_Dependency_Context() -> nomos_lang_go_modules::FactContext
{
    return nomos_lang_go_modules::FactContext {
        snapshot: SnapshotId::From_Digest(Content_Digest(b"nomos.determinism.snapshot")),
        variant: BuildVariantId::From_Digest(Content_Digest(b"nomos.determinism.variant")),
        configuration: ConfigurationId::From_Digest(Content_Digest(
            b"nomos.determinism.configuration",
        )),
        generation: GenerationId::INITIAL,
    };
}

/// The facts in package-name order, the identical reasoning
/// [`Rendered_Dependency_Facts`] gives for its own sort.
fn Rendered_Go_Dependency_Facts(mut facts: Vec<nomos_lang_go_modules::ModuleFact>) -> Vec<u8>
{
    facts.sort_by(|left, right| return left.fact.payload.bytes.cmp(&right.fact.payload.bytes));

    let mut rendered = Vec::new();
    for fact in &facts
    {
        rendered.extend_from_slice(format!("key\t{}\n", fact.fact.Key().Digest()).as_bytes());
        rendered.extend_from_slice(&fact.fact.payload.bytes);
    }

    return rendered;
}

/// The workspace root, from this crate's own manifest directory.
fn Repository_Root() -> std::path::PathBuf
{
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    return manifest
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::PathBuf::from)
        .expect("tests/integration sits two levels below the workspace root");
}

/// The fixture `nomos-lang-go`'s production is measured over.
///
/// A second, Go-specific fixture rather than the shared `FIXTURE` above — that one is Rust
/// source, and `nomos_lang_go::Read_Source` would refuse every file in it. Chosen for the
/// same reason `FIXTURE` was: constructs that have historically been a source of ordering
/// instability in an item walk. A receiver method (qualifies into its type's own scope, the
/// way an `impl` member does), an interface's method set (a second item per declaration,
/// recorded under the interface's name), and a multi-name grouped `const` block (repeats
/// the shared-field grammar quirk `Named_Field_Children` exists to filter).
const GO_FIXTURE: &[(&str, &str)] = &[
    (
        "held.go",
        "package held\n\n\
         // Held names something the package holds.\n\
         type Held struct {\n\
         \tfield int\n\
         }\n\n\
         func (h *Held) Build() *Held { return h }\n\n\
         func (h Held) Other() {}\n",
    ),
    (
        "writer.go",
        "package held\n\n\
         type Writer interface {\n\
         \tWrite(p []byte) (n int, err error)\n\
         \tio.Reader\n\
         }\n\n\
         const (\n\
         \tFirst, Second = 1, 2\n\
         \t// Third is three.\n\
         \tThird = 3\n\
         )\n",
    ),
    (
        "last.go",
        "package held\n\n\
         type Kind int\n\n\
         const (\n\
         \tFirst Kind = iota\n\
         \tSecond\n\
         )\n\n\
         var Count int\n\n\
         func last() {}\n",
    ),
];

fn Go_Subject_Of(path: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(path.as_bytes()));
}

fn Go_Context() -> nomos_lang_go::FactContext
{
    let shared = Fact_Context();

    return nomos_lang_go::FactContext {
        snapshot: shared.snapshot,
        variant: shared.variant,
        configuration: shared.configuration,
        generation: shared.generation,
    };
}

/// `nomos-lang-go`'s facts over [`GO_FIXTURE`], rendered the same way [`Parsed_Production`]
/// renders `nomos-lang-rust`'s.
pub(crate) fn Go_Production() -> Vec<u8>
{
    let context = Go_Context();
    let mut rendered = Vec::new();

    for (path, source) in GO_FIXTURE
    {
        let fact = match nomos_lang_go::Materialize_Syntax_Fact(Go_Subject_Of(path), source, context)
        {
            nomos_lang_go::Materialization::Materialized(fact) => fact,
            nomos_lang_go::Materialization::Unparseable(failure) =>
            {
                panic!("the fixture must parse; {path} did not: {failure}");
            }
        };

        rendered.extend_from_slice(format!("file\t{path}\n").as_bytes());
        rendered.extend_from_slice(format!("key\t{}\n", fact.Key().Digest()).as_bytes());
        rendered.extend_from_slice(&fact.payload.bytes);
    }

    return rendered;
}

/// The scanner's facts over the same fixture, rendered the same way.
pub(crate) fn Scanned_Production() -> Vec<u8>
{
    let context = Scan_Context();
    let mut rendered = Vec::new();

    for (path, source) in FIXTURE
    {
        let fact = nomos_lang_rust_scan::Materialize_Syntax_Fact(Subject_Of(path), source, context);
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
pub(crate) fn Reuse_Production() -> Vec<u8>
{
    let context = Fact_Context();
    let mut store = MemoryFactStore::New();
    let keys = Fill_With_Fixture(&mut store, context);
    let mut rendered = Vec::new();

    rendered.extend_from_slice(format!("live\t{}\n", store.Live()).as_bytes());
    rendered
        .extend_from_slice(format!("materializations\t{}\n", store.Materializations()).as_bytes());
    for identity in &keys
    {
        let outcome = Read_Outcome(&store, identity, context.generation);

        rendered.extend_from_slice(outcome.as_bytes());
        rendered.push(b'\n');
    }

    let invalidated = Invalidation_Report(&mut store);

    rendered.extend_from_slice(&invalidated);
    return rendered;
}

/// What the store answers for one key: a hit carrying its two digests, or a miss.
fn Read_Outcome(
    store: &MemoryFactStore,
    identity: &FactIdentity,
    generation: GenerationId,
) -> String
{
    let Some(fact) = store.Current(identity, generation)
    else
    {
        return format!("miss\t{}", identity.Key().Digest());
    };

    return format!("hit\t{}\t{}", fact.Key().Digest(), fact.payload.Digest());
}

/// What the store says it threw away when the configuration changes under it.
fn Invalidation_Report(store: &mut MemoryFactStore) -> Vec<u8>
{
    let report = store.Invalidate(
        &GenerationCause::ConfigurationChanged {
            configuration: ConfigurationId::From_Digest(Content_Digest(
                b"nomos.determinism.other",
            )),
        },
        GenerationId::INITIAL.Next(),
    );
    let mut rendered = Vec::new();

    rendered.extend_from_slice(format!("report\t{}\n", report.Report()).as_bytes());
    for key in &report.direct
    {
        rendered.extend_from_slice(format!("direct\t{}\n", key.Digest()).as_bytes());
    }

    return rendered;
}

/// The workspace snapshot's canonical bytes, after the fixture arrives through the door.
pub(crate) fn Snapshot_Production() -> Vec<u8>
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

/// A correction's whole lifecycle over a fresh workspace: previewed, staged, validated,
/// committed and rolled back.
///
/// Every stage's own identity is rendered rather than only the final state, because a
/// correction's promise is about the sequence — preview agreeing with what staging finds,
/// staging agreeing with what commit applies, and rollback exactly reversing it — not only
/// about where the workspace ends up. Ending back at the base snapshot is asserted directly
/// rather than only rendered, for the same reason `Assert_The_Rollup_Read_Its_Members`
/// asserts rather than trusts the bytes: a production that silently exercised less than it
/// claims would still repeat itself identically.
pub(crate) fn Correction_Production() -> Vec<u8>
{
    let mut workspace = Fresh_Workspace_With_The_Fixture_File();
    let plan = A_Plan_That_States_The_Return_Type();
    let base = workspace.Id();
    let mut rendered = Vec::new();

    rendered.extend_from_slice(b"preview\n");
    rendered.extend_from_slice(plan.Preview().Rendered());

    let staged = Staged_Against_The_Workspace(&plan, &workspace, &mut rendered);
    let committed = Validated_And_Committed(staged, &mut workspace, base, &mut rendered);
    Rolled_Back_To_The_Base(committed, &mut workspace, base, &mut rendered);

    return rendered;
}

/// A fresh workspace with the one file this lifecycle corrects already landed.
fn Fresh_Workspace_With_The_Fixture_File() -> Workspace
{
    let variant = BuildVariant::New("x86_64-unknown-none", "determinism", "fixed", ["one", "two"]);
    let configuration =
        ConfigurationId::From_Digest(Content_Digest(b"nomos.determinism.corrections"));
    let mut workspace = Workspace::Empty(variant, configuration);

    let presented = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("src/lib.rs", "pub fn a() {}");
    workspace.Apply(&presented).expect("a fresh present is always accepted");

    return workspace;
}

/// One candidate on one path: state the return type `a` takes for granted.
fn A_Plan_That_States_The_Return_Type() -> CorrectionPlan
{
    return CorrectionPlan::New(vec![CorrectionCandidate::New(
        "state the return type a takes for granted",
        ChangeSet::Empty().With(Edit::New(
            "src/lib.rs",
            Some("pub fn a() {}".to_owned()),
            Some("pub fn a() -> () {}".to_owned()),
        )),
        CorrectionClass::Mechanical,
        vec![],
    )])
    .expect("one candidate on one path is a valid plan");
}

/// The plan staged against the still-fresh workspace, with its base identity rendered —
/// staging agreeing with what commit applies is part of the sequence this production
/// exercises.
fn Staged_Against_The_Workspace(
    plan: &CorrectionPlan,
    workspace: &Workspace,
    rendered: &mut Vec<u8>,
) -> nomos_corrections::StagedPlan
{
    let staged = plan
        .Stage(workspace)
        .expect("the candidate's declared prior content matches the fixture");
    rendered.extend_from_slice(format!("staged-base\t{}\n", staged.Base()).as_bytes());

    return staged;
}

/// Validated against the still-unmoved workspace and then committed to it, with both
/// identities rendered and the commit asserted to have actually moved the workspace.
fn Validated_And_Committed(
    staged: nomos_corrections::StagedPlan,
    workspace: &mut Workspace,
    base: SnapshotId,
    rendered: &mut Vec<u8>,
) -> nomos_corrections::CommittedPlan
{
    let validated = staged
        .Validate(workspace)
        .expect("nothing has moved the workspace since staging");

    let evidence = Evidence {
        class: EvidenceClass::AgentJudged,
        producer: ProviderId::New("nomos-integration-tests.determinism"),
        supporting: Vec::new(),
    };
    let committed = validated
        .Commit(workspace, evidence)
        .expect("the workspace door accepts the forward change");
    rendered.extend_from_slice(format!("committed-base\t{}\n", committed.Base()).as_bytes());
    rendered.extend_from_slice(format!("committed-after\t{}\n", committed.After()).as_bytes());
    rendered.extend_from_slice(
        format!("committed-evidence-class\t{}\n", committed.Evidence().class).as_bytes(),
    );
    assert_ne!(committed.After(), base, "the commit must have changed the workspace");

    return committed;
}

/// Rolled back through the workspace door, with the identity it landed on rendered and
/// asserted to be exactly the snapshot the lifecycle started from.
fn Rolled_Back_To_The_Base(
    committed: nomos_corrections::CommittedPlan,
    workspace: &mut Workspace,
    base: SnapshotId,
    rendered: &mut Vec<u8>,
)
{
    let rolled_back_to = committed
        .Rollback(workspace)
        .expect("the workspace door accepts the reverse change");
    rendered.extend_from_slice(format!("rolled-back-to\t{rolled_back_to}\n").as_bytes());
    assert_eq!(
        rolled_back_to, base,
        "rolling back a commit must return to exactly the snapshot it started from"
    );
}
