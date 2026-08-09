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
    BuildVariantId, ConfigurationId, GenerationId, SnapshotId, Strategy, SubjectId,
};
use nomos_integration_tests::{
    Child_Variable, Cross_Environment_Owed, CrossEnvironment, Digest_In, Production, Report_Line,
    Verification, Verify,
};
use nomos_lang_rust::SyntaxFactProduction;
use nomos_lang_rust_scan::ScanFactProduction;
use nomos_model::Content_Digest;
use nomos_workspace::{
    BuildVariant, ChangeSource, SnapshotSerialization, Workspace, WorkspaceChangeSet,
};

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
const PARSED_GOLDEN: &str = "15cc14884df61a70872959a572706249";
const SCANNED_GOLDEN: &str = "94b9715ad47de8152fe3d3b2c53046ac";
const SNAPSHOT_GOLDEN: &str = "1fb5fb67d666b0bb983f3b71e7e09f93";

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
        "scan-fact-production" => "Test_The_Scanner_Should_Meet_Its_Declared_Strategy",
        "fact-reuse" => "Test_The_Fact_Cache_Should_Meet_Its_Declared_Strategy",
        "snapshot-serialization" => "Test_Snapshot_Serialization_Should_Meet_Its_Declared_Strategy",
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

// ---------------------------------------------------------------------------------
// The declarations themselves
// ---------------------------------------------------------------------------------

/// The domain table in `nomos_contracts::determinism` names six rows. Four are declared
/// in this workspace and this test says which two are not, so the gap is a statement
/// rather than an omission.
///
/// Spec-bundle serialization and the projection engine both live in `crates/spec`, which
/// `P9-DETERMINISM` did not claim and did not touch. Progress UI, logs and telemetry are
/// the `None` row and there is no such domain in the tree yet — the CLI prints, and
/// nothing about what it prints is a fact.
#[test]
fn Test_The_Declared_Domains_Should_Be_The_Ones_This_Item_Covered()
{
    let declared = [
        ("syntax-fact-production", SyntaxFactProduction::STRENGTH),
        ("scan-fact-production", ScanFactProduction::STRENGTH),
        ("fact-reuse", FactReuse::STRENGTH),
        ("snapshot-serialization", SnapshotSerialization::STRENGTH),
    ];

    assert_eq!(
        declared.len(),
        4,
        "four domains are covered; a fifth declaration needs a row in this table and a \
         test of its own"
    );

    for (domain, _) in declared
    {
        assert!(
            !Test_Name_For(domain).is_empty(),
            "{domain} declares a strategy and has no test registered"
        );
    }
}
