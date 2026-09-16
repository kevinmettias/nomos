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

/// The text half of a [`Source_File`]. A distinct type from the path half, so the two adjacent
/// string positions cannot be transposed at a call site and still compile.
struct SourceText<'a>(&'a str);

fn Source_File(path: &str, text: SourceText<'_>) -> SourceFile
{
    return SourceFile::New(path, Subject_Of_Path(path), text.0);
}

/// What one `Run` answered, reduced to the two halves this file compares.
struct Answer
{
    findings: Vec<Finding>,
    claim: Claim,
}

/// The three answers a class test compares: the run before its mutation, a run over the
/// mutated input that reuses the first run's workspace and store, and a clean recomputation
/// of that same input to measure the reuse against.
struct Comparison
{
    before: Answer,
    reused: Answer,
    clean: Answer,
}

/// The workspace and store a caller carries from one `Run` to the next, held together as one
/// value so a call site cannot thread half the pair to a different run than the other half.
struct Carried
{
    workspace: Option<Workspace>,
    store: MemoryFactStore,
}

impl Carried
{
    fn New() -> Self
    {
        return Self { workspace: None, store: MemoryFactStore::New() };
    }

    /// One `Run` over `root` with `sources`, reusing this value's own workspace and store,
    /// reduced to an [`Answer`]. Panics rather than returning an outcome: every call in this
    /// file is over a readable fixture, so a non-judged answer is a broken fixture rather
    /// than a result worth comparing.
    fn Answered(&mut self, sources: &[SourceFile], root: &Path, selected: &[RuleId]) -> Answer
    {
        let outcome = Run(
            sources,
            RunContext {
                variant: Test_Variant(),
                root,
                launcher: &StdProcessLauncher,
                filesystem: &StdFileSystem,
                environment: &StdEnvironment,
                workspace: &mut self.workspace,
                store: &mut self.store,
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
}

/// One `Run` over `root` with `sources`, over a workspace and store that have never been
/// used -- the clean recomputation every test below compares its reused run against.
fn Answered_Cleanly(sources: &[SourceFile], root: &Path, selected: &[RuleId]) -> Answer
{
    return Carried::New().Answered(sources, root, selected);
}

/// The source-derived class's shape: `before` judged once, then `after` judged again over the
/// same carried workspace and store, with a clean recomputation of `after` to compare against.
/// The mutation here is the caller's own source list, which `Run` sees directly.
fn Compare_Across_An_Edit(fixture: &Fixture, before: &[SourceFile], after: &[SourceFile], selected: &[RuleId]) -> Comparison
{
    let mut carried = Carried::New();

    let original = carried.Answered(before, fixture.Path(), selected);
    let reused = carried.Answered(after, fixture.Path(), selected);
    let clean = Answered_Cleanly(after, fixture.Path(), selected);

    return Comparison { before: original, reused, clean };
}

/// The shape the other two classes share: `sources` judged once, then a file under `fixture`'s
/// root that no caller passes rewritten by `mutate`, then the same sources judged again over
/// the carried workspace and store, and once more cleanly.
fn Compare_Across_A_Mutation(fixture: &Fixture, sources: &[SourceFile], selected: &[RuleId], mutate: impl FnOnce(&Fixture)) -> Comparison
{
    let mut carried = Carried::New();

    let before = carried.Answered(sources, fixture.Path(), selected);
    mutate(fixture);
    let reused = carried.Answered(sources, fixture.Path(), selected);
    let clean = Answered_Cleanly(sources, fixture.Path(), selected);

    return Comparison { before, reused, clean };
}

/// Asserts the property this file exists for, over the three answers a comparison holds.
fn Assert_Reuse_Agrees_With_Recomputation(comparison: &Comparison, class: &str)
{
    assert!(
        comparison.before.findings != comparison.reused.findings || comparison.before.claim != comparison.reused.claim,
        "{class}: the mutation must change what the run answers, or this fixture proves \
         nothing about invalidation -- recomputing everything and recomputing nothing both \
         pass an unchanged comparison"
    );
    assert_eq!(
        comparison.reused.claim, comparison.clean.claim,
        "{class}: a workspace and store reused across a mutation must reach the claim a clean \
         recomputation reaches"
    );
    assert_eq!(
        comparison.reused.findings, comparison.clean.findings,
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

    fn Write(&self, relative: &str, text: SourceText<'_>)
    {
        let path = self.root.join(relative);
        let parent = path.parent().expect("a joined fixture path names a file below its root");
        std::fs::create_dir_all(parent).expect("a fixture directory");
        std::fs::write(&path, text.0).expect("a fixture file");
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

/// The workspace-derived fixture's own manifests.
///
/// `violations.rs`'s own `Violations_In` produces no findings for a package the declaration
/// does not place -- `OD-RULES-029` measured why it must, since otherwise every member of
/// every other repository reads as a violation. A fixture whose packages are placed by
/// nothing therefore cannot produce a direction finding *at all*, in either generation, so an
/// edge added or dropped between them changes nothing a caller can observe and the comparison
/// below would assert nothing.
///
/// This used to be solved by naming the two members after real crates of *this* workspace, so
/// that the table compiled into `nomos-rules` would resolve them. That table is gone: the
/// declaration is now read from the root under check, so the fixture writes its own and the
/// names go back to `alpha` and `beta` like every other fixture in this file. The fixture is
/// better for it -- it no longer depends on which crates this repository happens to have.
const DEPENDENT_MANIFEST_WITH_EDGE: &str =
    "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nbeta = { path = \"../beta\" }\n";
const DEPENDENT_MANIFEST_WITHOUT_EDGE: &str = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const DEPENDED_MANIFEST: &str = "[package]\nname = \"beta\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// The architecture the workspace-derived fixture declares about itself.
///
/// Two components and one permitted direction: `beta` may name `alpha` and not the reverse, so
/// the edge `DEPENDENT_MANIFEST_WITH_EDGE` adds is exactly the violation the rule exists to
/// name, and dropping it is exactly the change the comparison needs to observe.
const FIXTURE_ARCHITECTURE: &str = "{\"components\":[\"Lower\",\"Upper\"],\"members\":{\"alpha\":\"Lower\",\"beta\":\"Upper\"},\"permits\":{\"Upper\":[\"Lower\"]}}\n";

/// The source-derived class: the input is the text a caller hands `Run`, and the mutation is
/// an edit to that text. This restates `run_context.rs`'s own internal proof from outside
/// the crate, against the public API a second adapter actually calls, so all three classes
/// are compared through one shape rather than two.
#[test]
fn Test_A_Source_Derived_Family_Reused_Across_An_Edit_Should_Agree_With_A_Clean_Recomputation()
{
    let fixture = Fixture::New("source");
    fixture.Write("Cargo.toml", SourceText(WORKSPACE_MANIFEST));

    let selected = [RuleId::New(nomos_rules::NAMING_CONVENTION)];
    let untouched = Source_File("b.rs", SourceText("pub fn Untouched() {}\n"));
    let unedited = [Source_File("a.rs", SourceText("pub fn Ok() {}\n")), untouched.clone()];
    let edited = [Source_File("a.rs", SourceText("pub fn Ok() {}\npub fn badly_named() {}\n")), untouched];

    let comparison = Compare_Across_An_Edit(&fixture, &unedited, &edited, &selected);

    Assert_Reuse_Agrees_With_Recomputation(&comparison, "source-derived");
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
    let selected = [RuleId::New(nomos_rules::DEPENDENCY_DIRECTION)];
    fixture.Write("Cargo.toml", SourceText(WORKSPACE_MANIFEST));
    fixture.Write("nomos-architecture.json", SourceText(FIXTURE_ARCHITECTURE));
    fixture.Write("alpha/Cargo.toml", SourceText(DEPENDENT_MANIFEST_WITH_EDGE));
    fixture.Write("alpha/src/lib.rs", SourceText("pub fn Ok() {}\n"));
    fixture.Write("beta/Cargo.toml", SourceText(DEPENDED_MANIFEST));
    fixture.Write("beta/src/lib.rs", SourceText("pub fn Ok() {}\n"));
    let sources = [Source_File("alpha/src/lib.rs", SourceText("pub fn Ok() {}\n"))];

    let comparison = Compare_Across_A_Mutation(&fixture, &sources, &selected, |fixture| {
        fixture.Write("alpha/Cargo.toml", SourceText(DEPENDENT_MANIFEST_WITHOUT_EDGE));
    });

    Assert_Reuse_Agrees_With_Recomputation(&comparison, "workspace-derived");
}

/// The repository-derived class: the input is a file under `root` read through the
/// `FileSystem` port, which no caller passes and no subprocess resolves, and the mutation
/// repoints one assessment's site at a path the tree does not hold.
#[test]
fn Test_A_Repository_Derived_Family_Reused_Across_A_Corpus_Edit_Should_Agree_With_A_Clean_Recomputation()
{
    let fixture = Fixture::New("repository");
    let selected = [RuleId::New(nomos_rules::REQUIREMENT_TRACE_STALENESS)];
    fixture.Write("Cargo.toml", SourceText(WORKSPACE_MANIFEST));
    fixture.Write("src/lib.rs", SourceText("pub fn Ok() {}\n"));

    let resolved = "verdict: Met\nrecord: OD-FIXTURE-001\nsite: src/lib.rs#Ok\n";
    let unresolved = "verdict: Met\nrecord: OD-FIXTURE-001\nsite: src/absent.rs#Gone\n";
    fixture.Write("tests/contract/requirements/CHK-001.assessment", SourceText(resolved));
    let sources = [Source_File("src/lib.rs", SourceText("pub fn Ok() {}\n"))];

    let comparison = Compare_Across_A_Mutation(&fixture, &sources, &selected, |fixture| {
        fixture.Write("tests/contract/requirements/CHK-001.assessment", SourceText(unresolved));
    });

    Assert_Reuse_Agrees_With_Recomputation(&comparison, "repository-derived");
}
