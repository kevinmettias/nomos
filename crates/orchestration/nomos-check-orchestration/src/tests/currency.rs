//! One proof per non-syntax family that its materializer skips an unchanged input and files
//! a changed one.
//!
//! `crate::facts::currency`'s own tests prove what the shared check is sensitive to, over a
//! fact built by hand. They cannot prove that a family's materializer routes through it, nor
//! that the family's own provider produces a *different* fact when its own input moves --
//! which is the half that decides whether a skip is economy or a stale answer, and is a
//! separate question per provider because no two of these read the same thing.
//!
//! Every test below therefore drives the real materializer three times over one store: a
//! first call that must file the fact, a second over an unmoved input that must file nothing,
//! and a third over a moved input that must file again. The third is the load-bearing one.
//! Every non-syntax provider in this workspace files with an empty `semantic_inputs`, so a
//! changed policy file, a changed manifest and a changed subprocess stream all produce a fact
//! under the *same* `nomos_analysis::FactKey` as the one before. A check comparing keys alone
//! would pass every second call here and fail every third.
//!
//! # Why one `Context` across all three calls
//!
//! Because that is the real case. `nomos_workspace::Workspace`'s own generation advances only
//! when a source a caller submitted moved, and none of the inputs mutated below is one: a
//! `standards.json`, a `Cargo.toml` and a `cargo clippy` stream are all read by a provider
//! from under `root`, never handed in as a `nomos_rules::SourceFile`. A fixture that advanced
//! the generation between calls would be testing a case a real `Run` cannot produce, and
//! would let a check keyed on the generation alone pass.
//!
//! # What the two subprocess families are driven through, and what that leaves unproven
//!
//! `nomos.cap.lint.diagnostics` and `nomos.cap.dependency.policy` are driven through a canned
//! launcher rather than a real `cargo clippy` or `cargo deny`, the same substitution those two
//! provider crates make in their own suites and for the same reason: a fabricated stream is
//! the only way to hold one input still and move another deliberately. `super::materialization`
//! is where the real subprocesses are exercised over this repository's own root.
//!
//! What that leaves unproven *here*, stated rather than implied: whether a real `cargo clippy`
//! or `cargo deny` run over an unchanged tree produces a byte-identical payload twice. If
//! either embeds anything that varies between runs, its currency check would never fire in
//! production and every test in this file would still pass.
//! `nomos.cap.dependency.edges` is proven against the real subprocess below, because `cargo
//! metadata` compiles nothing and is cheap enough to run twice.
//!
//! For the other two it is not asserted anywhere, and the nearest evidence is a reading rather
//! than an assertion:
//! `super::reuse::Test_A_Retained_Cache_Under_The_Production_Selection_Should_Skip_Most_Of_The_Composed_Table`
//! runs the whole composed table twice over this repository's own root, and its own doc records
//! that the second call runs *no* rule closure at all -- which it could not if either family
//! had reported itself changed. That reading is not a substitute for an assertion, because a
//! provider that refused outright would leave its family out of `changed` just as an unmoved
//! one does. A follow-up wanting the proof is paying for two more whole-workspace clippy runs
//! to get it.

use nomos_analysis::{Context, MemoryFactStore};
use nomos_platform::{Command, DeterminismStrength, ExitOutcome, ProgramLauncher, ProgramOutput};
use nomos_platform::{ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use std::path::{Path, PathBuf};

use crate::facts::{Materialize_Architecture, Materialize_Dependencies, Materialize_Goals_Policy};
use crate::facts::{Materialize_Limits_Policy, Materialize_Lint, Materialize_Naming_Policy};
use crate::facts::{Materialize_Policy, Materialize_Reachability, Materialize_Requirement_Trace};
use crate::facts::{Materialize_Scripting_Policy, Materialize_Test_Material_Policy};
use crate::facts::{Materialize_Words_Policy, Subprocess};

use super::{Repository_Root, Scratch_Directory, Source_File, SourceText, Test_Variant};

/// How many facts a two-member workspace's own `dependency.edges` materialization files on a
/// cold store: one per member, since `nomos_lang_rust_cargo` answers per member rather than
/// per workspace.
const WORKSPACE_MEMBERS: u32 = 2;

/// The one fact a whole-workspace or per-source materialization files on a cold store.
const ONE_FACT: u32 = 1;

/// The `Context` every test here files under: real, ingested, and built from a placeholder
/// source because none of these materializations reads `sources` at all -- the identical
/// reason [`super::materialization`]'s own `Ingested_Placeholder` gives.
fn Fixture_Context() -> Context
{
    let placeholder = [Source_File("placeholder.rs", SourceText("pub fn Placeholder() {}\n"))];
    let registry = crate::composition::Registered().expect("fixture composition");

    return crate::facts::Ingested_Workspace(&placeholder, &registry, Test_Variant(), &mut None)
        .expect("a single real file ingests");
}

fn Write_Fixture_File(root: &Path, relative: &str, text: &str)
{
    let path = root.join(relative);
    let parent = path.parent().expect("a joined fixture path names a file below its root");
    std::fs::create_dir_all(parent).expect("a fixture directory");
    std::fs::write(&path, text).expect("a fixture file");
}

/// The three counts one currency proof reads off a store, named so a caller comparing them
/// cannot read one for another.
struct FilingCounts
{
    cold: u32,
    unmoved: u32,
    moved: u32,
}

/// Asserts the shape every test in this file claims, quoting `family` in each failure: a cold
/// store is filed into, an unmoved input files nothing at all, and a moved input files again.
fn Assert_Currency_Proven(counts: &FilingCounts, family: &str, expected_cold: u32)
{
    assert_eq!(
        counts.cold, expected_cold,
        "{family}: a cold store must be filed into, or the two comparisons below compare nothing"
    );
    assert_eq!(
        counts.unmoved, 0,
        "{family}: a second call over an unmoved input filed {} fact(s). The provider answered exactly what the \
         store was already serving, so the write was redundant -- and `Materialization_Tracking` reads the \
         counter's move as this family having changed, which re-judges every rule declaring it",
        counts.unmoved
    );
    assert!(
        counts.moved > 0,
        "{family}: a call over a *moved* input filed nothing, so the check is skipping writes whose inputs have \
         genuinely changed and a reused store now answers from a stale fact. Every non-syntax provider here \
         files with an empty `semantic_inputs`, so this is exactly what a key-only comparison looks like"
    );
}

/// One repository-declared policy family, as one row of the table below.
struct PolicyFamilyCase
{
    family: &'static str,
    materialize: fn(&Path, &Context, &mut MemoryFactStore, &StdFileSystem) -> usize,
    /// The file under `root` this family's own provider reads.
    file: &'static str,
    declared: &'static str,
    redeclared: &'static str,
}

/// The eight repository-declared policy families, each with the file its own provider reads
/// and two declarations that must produce different facts.
///
/// `file` is the fixture's knowledge and not a second authority over what a provider reads:
/// nothing in `crate::facts` names any of these paths, and a row naming the wrong one fails
/// here on the moved call rather than silently agreeing with the code it checks.
fn Policy_Family_Cases() -> Vec<PolicyFamilyCase>
{
    return vec![
        PolicyFamilyCase { family: "naming.policy", materialize: Materialize_Naming_Policy, file: STANDARDS_JSON,
            declared: r#"{"naming":{"function":"lower-snake"}}"#, redeclared: r#"{"naming":{"function":"upper-snake"}}"# },
        PolicyFamilyCase { family: "limits.policy", materialize: Materialize_Limits_Policy, file: STANDARDS_JSON,
            declared: r#"{"limits":{"file-size-hard-lines":3}}"#, redeclared: r#"{"limits":{"file-size-hard-lines":4}}"# },
        PolicyFamilyCase { family: "scripting.policy", materialize: Materialize_Scripting_Policy, file: STANDARDS_JSON,
            declared: r#"{"scripting":{"tooling_language":"rust","forbidden_extensions":[".sh"]}}"#,
            redeclared: r#"{"scripting":{"tooling_language":"rust","forbidden_extensions":[".bat"]}}"# },
        PolicyFamilyCase { family: "goals.policy", materialize: Materialize_Goals_Policy, file: STANDARDS_JSON,
            declared: r#"{"goals":["render"]}"#, redeclared: r#"{"goals":["parse"]}"# },
        PolicyFamilyCase { family: "words.policy", materialize: Materialize_Words_Policy, file: STANDARDS_JSON,
            declared: r#"{"words":{"approved_abbreviations":["ctx"]}}"#, redeclared: r#"{"words":{"approved_abbreviations":["cfg"]}}"# },
        PolicyFamilyCase { family: "architecture.declaration", materialize: Materialize_Architecture, file: "nomos-architecture.json",
            declared: FIXTURE_ARCHITECTURE, redeclared: REDECLARED_FIXTURE_ARCHITECTURE },
        PolicyFamilyCase { family: "test.material.policy", materialize: Materialize_Test_Material_Policy, file: "nomos-test-material.json",
            declared: r#"{"fixture_locations":["fixtures/"]}"#, redeclared: r#"{"fixture_locations":["corpora/"]}"# },
        PolicyFamilyCase { family: "requirement.trace", materialize: Materialize_Requirement_Trace, file: FIXTURE_ASSESSMENT,
            declared: "verdict: Met\nrecord: OD-FIXTURE-001\nsite: src/lib.rs#Ok\n",
            redeclared: "verdict: Met\nrecord: OD-FIXTURE-001\nsite: src/absent.rs#Gone\n" },
    ];
}

/// The file five of the eight families read, named once because five rows spell it.
const STANDARDS_JSON: &str = "standards.json";

/// The one assessment the requirement-trace family's fixture declares. That family reads a
/// directory rather than a file, so the mutation below rewrites the one entry in it.
const FIXTURE_ASSESSMENT: &str = "tests/contract/requirements/CHK-001.assessment";

/// Two components and one permitted direction, and the same declaration with that direction
/// dropped -- the smallest pair whose encoded payloads must differ.
const FIXTURE_ARCHITECTURE: &str = r#"{"components":["Lower","Upper"],"members":{"alpha":"Lower","beta":"Upper"},"permits":{"Upper":["Lower"]}}"#;
const REDECLARED_FIXTURE_ARCHITECTURE: &str = r#"{"components":["Lower","Upper"],"members":{"alpha":"Lower","beta":"Upper"},"permits":{}}"#;

/// Every repository-declared policy family proves its own currency, over its own real
/// provider and its own real file on disk.
///
/// One test over eight rows rather than eight tests, because the eight differ only in which
/// provider they wrap and which file it reads -- the identical reason
/// `crate::facts::policy_materialization`'s own module doc gives for driving them from one
/// list. A row that stopped being proven names itself in the failure.
#[test]
fn Test_Every_Policy_Family_Should_Skip_An_Unchanged_Declaration_And_File_A_Changed_One()
{
    for case in Policy_Family_Cases()
    {
        Assert_Policy_Family_Proves_Currency(&case);
    }
}

fn Assert_Policy_Family_Proves_Currency(case: &PolicyFamilyCase)
{
    let root = Scratch_Directory(&format!("currency-{}", case.family));
    let context = Fixture_Context();
    let mut store = MemoryFactStore::New();
    Write_Fixture_File(&root, case.file, case.declared);

    let cold = Filed_By(&mut store, |store| { (case.materialize)(&root, &context, store, &StdFileSystem); });
    let unmoved = Filed_By(&mut store, |store| { (case.materialize)(&root, &context, store, &StdFileSystem); });
    Write_Fixture_File(&root, case.file, case.redeclared);
    let moved = Filed_By(&mut store, |store| { (case.materialize)(&root, &context, store, &StdFileSystem); });

    Assert_Currency_Proven(&FilingCounts { cold, unmoved, moved }, case.family, ONE_FACT);
}

/// How many facts `store` gained while `materialize` ran -- the observable every proof here
/// reads, because a materializer's own return value answers coverage and is `1` for a skip
/// and for a write alike.
fn Filed_By(store: &mut MemoryFactStore, materialize: impl FnOnce(&mut MemoryFactStore)) -> u32
{
    let before = store.Materializations();
    materialize(store);

    return store.Materializations().saturating_sub(before);
}

/// The reachability family, whose input is the source text a caller hands in.
///
/// The one non-syntax family whose provider already files a digest of its own input in the
/// fact's key, so its second call would be skipped by a key comparison alone. It is proven
/// here anyway, and by the same shape as the seven that would not be, because what is under
/// test is that its write routes through the check at all.
#[test]
fn Test_The_Reachability_Family_Should_Skip_An_Unchanged_Source_And_File_A_Changed_One()
{
    let context = Fixture_Context();
    let mut store = MemoryFactStore::New();
    let unmoved_source = [Source_File("a.rs", SourceText("pub fn One() -> u8\n{\n    return 1;\n}\n"))];
    let moved_source = [Source_File("a.rs", SourceText("pub fn One() -> u8\n{\n    return 2;\n}\n"))];

    let cold = Filed_By(&mut store, |store| { Materialize_Reachability(&unmoved_source, &context, store); });
    let unmoved = Filed_By(&mut store, |store| { Materialize_Reachability(&unmoved_source, &context, store); });
    let moved = Filed_By(&mut store, |store| { Materialize_Reachability(&moved_source, &context, store); });

    Assert_Currency_Proven(&FilingCounts { cold, unmoved, moved }, "controlflow.reachability", ONE_FACT);
}

const WORKSPACE_MANIFEST: &str = "[workspace]\nmembers = [\"alpha\", \"beta\"]\nresolver = \"2\"\n";
const DEPENDENT_MANIFEST_WITH_EDGE: &str =
    "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nbeta = { path = \"../beta\" }\n";
const DEPENDENT_MANIFEST_WITHOUT_EDGE: &str = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const DEPENDED_MANIFEST: &str = "[package]\nname = \"beta\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// A two-member Cargo workspace on disk, which is what `cargo metadata` resolves.
fn Dependency_Fixture_Root(name: &str) -> PathBuf
{
    let root = Scratch_Directory(name);
    Write_Fixture_File(&root, "Cargo.toml", WORKSPACE_MANIFEST);
    Write_Fixture_File(&root, "alpha/Cargo.toml", DEPENDENT_MANIFEST_WITH_EDGE);
    Write_Fixture_File(&root, "alpha/src/lib.rs", "pub fn Ok() {}\n");
    Write_Fixture_File(&root, "beta/Cargo.toml", DEPENDED_MANIFEST);
    Write_Fixture_File(&root, "beta/src/lib.rs", "pub fn Ok() {}\n");

    return root;
}

/// The dependency-edges family against a real `cargo metadata`, over a real two-member
/// workspace whose dependent manifest is rewritten between the second call and the third.
///
/// Real rather than canned, unlike the two subprocess families below, for the reason this
/// file's own doc gives: `cargo metadata` compiles nothing, so this is the one place a real
/// subprocess's own output stability is affordable to check. The moved call proves the edit
/// reached the fact; the unmoved one proves two real resolutions of the same tree produce the
/// same payload, which is the property a canned stream cannot speak to.
#[test]
fn Test_The_Dependency_Edges_Family_Should_Skip_An_Unchanged_Manifest_And_File_A_Changed_One()
{
    let root = Dependency_Fixture_Root("currency-dependency-edges");
    let context = Fixture_Context();
    let mut store = MemoryFactStore::New();

    let cold = Filed_By(&mut store, |store| { Materialized_Edges(&root, &context, store); });
    let unmoved = Filed_By(&mut store, |store| { Materialized_Edges(&root, &context, store); });
    Write_Fixture_File(&root, "alpha/Cargo.toml", DEPENDENT_MANIFEST_WITHOUT_EDGE);
    let moved = Filed_By(&mut store, |store| { Materialized_Edges(&root, &context, store); });

    Assert_Currency_Proven(&FilingCounts { cold, unmoved, moved }, "dependency.edges", WORKSPACE_MEMBERS);
}

/// One real `Materialize_Dependencies` call, with its own findings asserted empty -- a real
/// workspace root must not report the provider unavailable, which is what makes the counts
/// above the provider's own answer rather than a refusal's.
fn Materialized_Edges(root: &Path, context: &Context, store: &mut MemoryFactStore)
{
    let materialized = Materialize_Dependencies(root, context, store, Subprocess { launcher: &StdProgramLauncher, environment: &StdEnvironment });

    assert!(materialized.findings.is_empty(), "a real two-member workspace must resolve: {:?}", materialized.findings);
}

/// A launcher answering one fixed stream, so a test can hold a provider's input still and
/// then move it deliberately -- the shape `nomos_lang_rust_clippy`'s and
/// `nomos_lang_rust_deny`'s own suites each reimplement for the identical reason.
struct CannedLauncher
{
    stdout: String,
    stderr: String,
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for CannedLauncher
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for CannedLauncher
{
    fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
    {
        return Ok(ProgramOutput { outcome: ExitOutcome::Exited { code: 0 }, stdout: self.stdout.clone(), stderr: self.stderr.clone() });
    }
}

/// The member every canned clippy stream attributes its diagnostic to.
const LINT_MEMBER: &str = "alpha";

/// Where a substitution goes in the fixture streams below, spelled once rather than at each
/// `replace` call.
const PACKAGE_PLACEHOLDER: &str = "THE-PACKAGE-ID";
const MESSAGE_PLACEHOLDER: &str = "THE-MESSAGE";

/// The `compiler-artifact` and `compiler-message` pair `cargo clippy --message-format=json`
/// prints for one member reporting one lint, as two JSON lines.
const CLIPPY_ARTIFACT_LINE: &str = r#"{"reason":"compiler-artifact","package_id":"THE-PACKAGE-ID","target":{"kind":["lib"]}}"#;
const CLIPPY_MESSAGE_LINE: &str = r#"{"reason":"compiler-message","package_id":"THE-PACKAGE-ID","message":{"level":"warning","message":"THE-MESSAGE","code":{"code":"clippy::needless_return"},"spans":[{"file_name":"src/lib.rs","line_start":5,"is_primary":true}]}}"#;

/// A `path+file://` package id `cargo clippy` could plausibly report for `path` -- the same
/// construction `nomos_lang_rust_clippy`'s own suite builds, reproduced because that one is
/// private to it.
fn Package_Id_Uri(path: &Path) -> String
{
    let forward = path.to_string_lossy().replace('\\', "/");
    if forward.starts_with('/')
    {
        return format!("path+file://{forward}#0.1.0");
    }

    return format!("path+file:///{forward}#0.1.0");
}

/// One member's clippy stream, reporting `message` as its own one diagnostic.
fn Clippy_Stream(root: &Path, message: &str) -> String
{
    let package = Package_Id_Uri(&root.join(LINT_MEMBER));
    let artifact = CLIPPY_ARTIFACT_LINE.replace(PACKAGE_PLACEHOLDER, &package);
    let diagnostic = CLIPPY_MESSAGE_LINE.replace(PACKAGE_PLACEHOLDER, &package).replace(MESSAGE_PLACEHOLDER, message);

    return format!("{artifact}\n{diagnostic}\n");
}

/// The lint-diagnostics family: an unchanged `cargo clippy` stream files nothing a second
/// time, and one reporting a different diagnostic files again.
#[test]
fn Test_The_Lint_Family_Should_Skip_An_Unchanged_Diagnostic_Stream_And_File_A_Changed_One()
{
    let root = Scratch_Directory("currency-lint");
    let context = Fixture_Context();
    let mut store = MemoryFactStore::New();
    let unmoved_stream = Clippy_Stream(&root, "unneeded return statement");
    let moved_stream = Clippy_Stream(&root, "this expression creates a reference which is immediately dereferenced");

    let cold = Filed_By(&mut store, |store| { Materialized_Lint(&root, &context, store, &unmoved_stream); });
    let unmoved = Filed_By(&mut store, |store| { Materialized_Lint(&root, &context, store, &unmoved_stream); });
    let moved = Filed_By(&mut store, |store| { Materialized_Lint(&root, &context, store, &moved_stream); });

    Assert_Currency_Proven(&FilingCounts { cold, unmoved, moved }, "lint.diagnostics", ONE_FACT);
}

fn Materialized_Lint(root: &Path, context: &Context, store: &mut MemoryFactStore, stdout: &str)
{
    let launcher = CannedLauncher { stdout: stdout.to_owned(), stderr: String::new() };
    let materialized = Materialize_Lint(root, context, store, Subprocess { launcher: &launcher, environment: &StdEnvironment });

    assert!(materialized.findings.is_empty(), "a readable clippy stream must not report the provider unavailable: {:?}", materialized.findings);
}

/// One `cargo deny` diagnostic and the summary line a finished run always writes, as the two
/// JSON lines that provider reads off **stderr**.
const DENY_DIAGNOSTIC_LINE: &str = r#"{"type":"diagnostic","fields":{"code":"duplicate","severity":"warning","message":"THE-MESSAGE","labels":[],"graphs":[]}}"#;
const DENY_SUMMARY_LINE: &str = r#"{"type":"summary","fields":{"bans":{"errors":0,"helps":0,"notes":0,"warnings":0},"licenses":{"errors":0,"helps":0,"notes":0,"warnings":0},"sources":{"errors":0,"helps":0,"notes":0,"warnings":0}}}"#;

fn Deny_Stream(message: &str) -> String
{
    return format!("{}\n{}\n", DENY_DIAGNOSTIC_LINE.replace(MESSAGE_PLACEHOLDER, message), DENY_SUMMARY_LINE);
}

/// The dependency-policy family: an unchanged `cargo deny` stream files nothing a second
/// time, and one reporting a different violation files again.
///
/// A whole-workspace capability, so its check is all-or-nothing rather than per subject --
/// `IncrementalGranularity::WholeWorkspace` showing through, since a bans/licenses/sources
/// verdict is not attributable to one member.
#[test]
fn Test_The_Dependency_Policy_Family_Should_Skip_An_Unchanged_Verdict_Stream_And_File_A_Changed_One()
{
    let root = Scratch_Directory("currency-dependency-policy");
    // `P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT`: this provider refuses before reaching
    // any launcher, real or canned, unless `root` carries its own `deny.toml`.
    Write_Fixture_File(&root, "deny.toml", "[bans]\nmultiple-versions = \"warn\"\n");
    let context = Fixture_Context();
    let mut store = MemoryFactStore::New();
    let unmoved_stream = Deny_Stream("multiple versions of a crate are present");
    let moved_stream = Deny_Stream("a crate is banned by this repository's own declaration");

    let cold = Filed_By(&mut store, |store| { Materialized_Dependency_Policy(&root, &context, store, &unmoved_stream); });
    let unmoved = Filed_By(&mut store, |store| { Materialized_Dependency_Policy(&root, &context, store, &unmoved_stream); });
    let moved = Filed_By(&mut store, |store| { Materialized_Dependency_Policy(&root, &context, store, &moved_stream); });

    Assert_Currency_Proven(&FilingCounts { cold, unmoved, moved }, "dependency.policy", ONE_FACT);
}

fn Materialized_Dependency_Policy(root: &Path, context: &Context, store: &mut MemoryFactStore, stderr: &str)
{
    let launcher = CannedLauncher { stdout: String::new(), stderr: stderr.to_owned() };
    let materialized = Materialize_Policy(root, context, store, Subprocess { launcher: &launcher, environment: &StdEnvironment });

    assert!(materialized.findings.is_empty(), "a readable deny stream must not report the provider unavailable: {:?}", materialized.findings);
}

/// The one claim about a real subprocess this file makes beyond `dependency.edges`' own
/// fixture: two real `cargo metadata` resolutions of *this repository's* own root, over one
/// store, file the second nothing.
///
/// Separate from the fixture test above because it answers a different question. That one
/// proves an edit reaches the fact, over a tree small enough to rewrite. This one proves the
/// payload is stable over a real, large, dependency-resolving workspace -- the tree a real
/// `nomos check` runs against, where a resolution order or a path rendering that varied
/// between runs would make the skip never fire.
#[test]
fn Test_A_Real_Repository_Resolution_Should_File_Nothing_The_Second_Time()
{
    let root = Repository_Root();
    let context = Fixture_Context();
    let mut store = MemoryFactStore::New();

    let cold = Filed_By(&mut store, |store| { Materialized_Edges(&root, &context, store); });
    let unmoved = Filed_By(&mut store, |store| { Materialized_Edges(&root, &context, store); });

    assert!(cold > WORKSPACE_MEMBERS, "this repository has far more than {WORKSPACE_MEMBERS} members, so {cold} facts is too few to have reached the provider");
    assert_eq!(unmoved, 0, "a second real resolution of an unchanged workspace filed {unmoved} fact(s), so `cargo metadata`'s own payload is not stable across two runs and no currency check on this family can ever fire");
}
