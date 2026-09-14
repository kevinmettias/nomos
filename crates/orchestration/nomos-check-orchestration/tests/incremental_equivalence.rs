//! Reuse-equals-recomputation, through the real `Run`, across the three classes of input a
//! materialized fact can come from.
//!
//! `nomos-analysis`'s own `tests/recomputation_equivalence.rs` proves this property on a
//! synthetic fixture built directly on `MemoryFactStore`'s `Materialize`/`Invalidate`/
//! `Current` API, and its module doc states plainly that it exercises store mechanics
//! rather than any capability's business logic. `nomos-lang-rust-cargo`'s own
//! `tests/invalidation.rs` proves an edited manifest's old edge fact does not survive the
//! generation it was invalidated at, but never compares a reused run against a clean one.
//! `run_context.rs`'s own `Test_A_Store_And_Workspace_Reused_Across_An_Edit_Agrees_With_A_
//! Clean_Recomputation` is the only test that asserts the property through `Run` itself,
//! and it selects one rule -- `NAMING_CONVENTION`, feeding on `SyntaxItems` and
//! `NamingPolicy`, both derived from bytes the caller hands in.
//!
//! `Materialize_Capabilities` materializes twelve `RequiredFact` families, and they do not
//! share one input. Three classes do the real dividing:
//!
//! - **source-derived**: the input is the `SourceFile` text a caller passes. `Run` sees the
//!   mutation directly.
//! - **workspace-derived**: the input is a manifest tree under `root` read by a subprocess.
//!   No caller passes it, and `Run` learns of the mutation only by resolving it again.
//! - **repository-derived**: the input is a file under `root` read through the `FileSystem`
//!   port. No caller passes it and no subprocess resolves it.
//!
//! An under-invalidating family in either of the last two serves a stale fact, which is a
//! wrong answer rather than a missing one, and it is silent in exactly the conditions this
//! suite runs under. One test per class below, each mutating that class's own real input.
//!
//! Every test asserts two things, and the second is what keeps the first from being
//! vacuous: that a reused workspace and store agree with a clean recomputation, and that
//! the mutation was observable at all -- a fixture whose two generations already agree
//! proves nothing about invalidation, since recomputing everything and recomputing nothing
//! both pass that comparison.

use nomos_analysis::MemoryFactStore;
use nomos_check_orchestration::{CheckOutcome, Claim, Run, RunContext};
use nomos_contracts::{Finding, RuleId};
use nomos_model::Subject_Of_Path;
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::{BuildVariant, Workspace};
use std::path::{Path, PathBuf};

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

fn Source(path: &str, text: &str) -> SourceFile
{
    return SourceFile::New(path, Subject_Of_Path(path), text);
}

/// What one `Run` answered, reduced to the two halves this file compares.
struct Answer
{
    findings: Vec<Finding>,
    claim: Claim,
}

/// One `Run` over `root` with `sources`, reusing whatever `workspace` and `store` are handed
/// in, reduced to an [`Answer`]. Panics rather than returning an outcome: every call in this
/// file is over a readable fixture, so a non-judged answer is a broken fixture rather than a
/// result worth comparing.
fn Answered(
    sources: &[SourceFile], root: &Path, selected: &[RuleId], workspace: &mut Option<Workspace>, store: &mut MemoryFactStore,
) -> Answer
{
    let outcome = Run(
        sources,
        RunContext {
            variant: Test_Variant(),
            root,
            launcher: &StdProcessLauncher,
            filesystem: &StdFileSystem,
            environment: &StdEnvironment,
            workspace,
            store,
        },
        selected,
    );

    let CheckOutcome::Judged { findings, claim, .. } = outcome
    else
    {
        panic!("a run over a readable fixture must be judged");
    };

    return Answer { findings, claim };
}

/// One `Run` over `root` with `sources`, over a workspace and store that have never been
/// used -- the clean recomputation every test below compares its reused run against.
fn Answered_Cleanly(sources: &[SourceFile], root: &Path, selected: &[RuleId]) -> Answer
{
    return Answered(sources, root, selected, &mut None, &mut MemoryFactStore::New());
}

/// Asserts the property this file exists for, over the three answers a test produced.
fn Assert_Reuse_Agrees_With_Recomputation(before: &Answer, reused: &Answer, clean: &Answer, class: &str)
{
    assert!(
        before.findings != reused.findings || before.claim != reused.claim,
        "{class}: the mutation must change what the run answers, or this fixture proves \
         nothing about invalidation -- recomputing everything and recomputing nothing both \
         pass an unchanged comparison"
    );
    assert_eq!(
        reused.claim, clean.claim,
        "{class}: a workspace and store reused across a mutation must reach the claim a clean \
         recomputation reaches"
    );
    assert_eq!(
        reused.findings, clean.findings,
        "{class}: a workspace and store reused across a mutation must report the findings a \
         clean recomputation reports"
    );
}

/// A scratch tree, removed when the test that built it ends.
struct Fixture
{
    root: PathBuf,
}

impl Fixture
{
    fn New(name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-equivalence-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);

        return Self { root };
    }

    fn Write(&self, relative: &str, text: &str)
    {
        let path = self.root.join(relative);
        let parent = path.parent().expect("a fixture path always has a parent");
        std::fs::create_dir_all(parent).expect("a fixture directory");
        std::fs::write(&path, text).expect("a fixture file");
    }

    fn Path(&self) -> &Path
    {
        return &self.root;
    }
}

impl Drop for Fixture
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

const WORKSPACE_MANIFEST: &str = "[workspace]\nmembers = [\"alpha\", \"beta\"]\nresolver = \"2\"\n";

/// The workspace-derived fixture's own manifests, and why its two members are named after
/// real crates of *this* workspace rather than `alpha` and `beta` like every other fixture
/// in this file.
///
/// `nomos_rules::checks::dependency::zones::ZONES` is a table compiled into `nomos-rules`,
/// and `violations.rs`'s own `Violations_In` silently produces no findings for a package
/// with no declared zone -- `OD-RULES-029` measured why it must, since otherwise every
/// member of every other repository reads as a violation. A fixture whose packages are
/// named `alpha` and `beta` therefore cannot produce a direction finding *at all*, in
/// either generation, so an edge added or dropped between them changes nothing a caller can
/// observe and the comparison below would assert nothing. `nomos-contracts` is declared
/// `Zone::Protocol` and `nomos-cli` is declared `Zone::Host`, so an edge from the first to
/// the second runs strictly upward and is exactly the violation this rule exists to name.
const DEPENDENT_MANIFEST_WITH_EDGE: &str =
    "[package]\nname = \"nomos-contracts\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nnomos-cli = { path = \"../beta\" }\n";
const DEPENDENT_MANIFEST_WITHOUT_EDGE: &str = "[package]\nname = \"nomos-contracts\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const DEPENDED_MANIFEST: &str = "[package]\nname = \"nomos-cli\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// The source-derived class: the input is the text a caller hands `Run`, and the mutation is
/// an edit to that text. This restates `run_context.rs`'s own internal proof from outside
/// the crate, against the public API a second adapter actually calls, so all three classes
/// are compared through one shape rather than two.
#[test]
fn Test_A_Source_Derived_Family_Reused_Across_An_Edit_Should_Agree_With_A_Clean_Recomputation()
{
    let fixture = Fixture::New("source");
    fixture.Write("Cargo.toml", WORKSPACE_MANIFEST);

    let selected = [RuleId::New(nomos_rules::NAMING_CONVENTION)];
    let untouched = Source("b.rs", "pub fn Untouched() {}\n");
    let unedited = [Source("a.rs", "pub fn Ok() {}\n"), untouched.clone()];
    let edited = [Source("a.rs", "pub fn Ok() {}\npub fn badly_named() {}\n"), untouched];

    let mut workspace = None;
    let mut store = MemoryFactStore::New();
    let before = Answered(&unedited, fixture.Path(), &selected, &mut workspace, &mut store);
    let reused = Answered(&edited, fixture.Path(), &selected, &mut workspace, &mut store);
    let clean = Answered_Cleanly(&edited, fixture.Path(), &selected);

    Assert_Reuse_Agrees_With_Recomputation(&before, &reused, &clean, "source-derived");
}

/// The workspace-derived class: the input is a manifest tree under `root` that no caller
/// passes, resolved by a real subprocess, and the mutation is a rewritten manifest dropping
/// one edge. `nomos-lang-rust-cargo`'s own `tests/invalidation.rs` proves the old edge fact
/// does not survive the generation it was invalidated at; this proves the run built on top
/// of it answers what a clean run answers.
#[test]
fn Test_A_Workspace_Derived_Family_Reused_Across_A_Manifest_Edit_Should_Agree_With_A_Clean_Recomputation()
{
    let fixture = Fixture::New("workspace");
    fixture.Write("Cargo.toml", WORKSPACE_MANIFEST);
    fixture.Write("alpha/Cargo.toml", DEPENDENT_MANIFEST_WITH_EDGE);
    fixture.Write("alpha/src/lib.rs", "pub fn Ok() {}\n");
    fixture.Write("beta/Cargo.toml", DEPENDED_MANIFEST);
    fixture.Write("beta/src/lib.rs", "pub fn Ok() {}\n");

    let selected = [RuleId::New(nomos_rules::DEPENDENCY_DIRECTION)];
    let sources = [Source("alpha/src/lib.rs", "pub fn Ok() {}\n")];

    let mut workspace = None;
    let mut store = MemoryFactStore::New();
    let before = Answered(&sources, fixture.Path(), &selected, &mut workspace, &mut store);

    fixture.Write("alpha/Cargo.toml", DEPENDENT_MANIFEST_WITHOUT_EDGE);

    let reused = Answered(&sources, fixture.Path(), &selected, &mut workspace, &mut store);
    let clean = Answered_Cleanly(&sources, fixture.Path(), &selected);

    Assert_Reuse_Agrees_With_Recomputation(&before, &reused, &clean, "workspace-derived");
}

/// The repository-derived class: the input is a file under `root` read through the
/// `FileSystem` port, which no caller passes and no subprocess resolves, and the mutation
/// repoints one assessment's site at a path the tree does not hold.
#[test]
fn Test_A_Repository_Derived_Family_Reused_Across_A_Corpus_Edit_Should_Agree_With_A_Clean_Recomputation()
{
    let fixture = Fixture::New("repository");
    fixture.Write("Cargo.toml", WORKSPACE_MANIFEST);
    fixture.Write("src/lib.rs", "pub fn Ok() {}\n");

    let resolved = "verdict: Met\nrecord: OD-FIXTURE-001\nsite: src/lib.rs#Ok\n";
    let unresolved = "verdict: Met\nrecord: OD-FIXTURE-001\nsite: src/absent.rs#Gone\n";
    fixture.Write("tests/contract/requirements/CHK-001.assessment", resolved);

    let selected = [RuleId::New(nomos_rules::REQUIREMENT_TRACE_STALENESS)];
    let sources = [Source("src/lib.rs", "pub fn Ok() {}\n")];

    let mut workspace = None;
    let mut store = MemoryFactStore::New();
    let before = Answered(&sources, fixture.Path(), &selected, &mut workspace, &mut store);

    fixture.Write("tests/contract/requirements/CHK-001.assessment", unresolved);

    let reused = Answered(&sources, fixture.Path(), &selected, &mut workspace, &mut store);
    let clean = Answered_Cleanly(&sources, fixture.Path(), &selected);

    Assert_Reuse_Agrees_With_Recomputation(&before, &reused, &clean, "repository-derived");
}
