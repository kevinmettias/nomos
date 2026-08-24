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
use nomos_corrections::{ChangeSet, CorrectionCandidate, CorrectionPlan, Edit};
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
    let fact = match nomos_lang_rust::Materialize(Subject_Of(path), source, context)
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
    let fact = match nomos_lang_rust::reachability::Materialize(Subject_Of(path), source, context)
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
            .map(|(path, source)| return rollup::ModuleMember::Of(Subject_Of(path), source))
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
            nomos_lang_rust::Materialize(Subject_Of(path), source, context)
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

/// The scanner's facts over the same fixture, rendered the same way.
pub(crate) fn Scanned_Production() -> Vec<u8>
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
