//! What this crate promises, exercised directly against [`crate::Run`] and, for the one
//! guarantee `Run`'s single entry point cannot isolate on its own, against its private
//! pieces.
//!
//! The composition here is the real one -- the real syntax provider, the real rule -- over
//! sources a test wrote by hand, which is what makes every assertion a statement about the
//! shipped seam rather than about a fixture. `nomos-cli`'s own `check` suite exercises the
//! same paths again through the compiled binary; this file is the crate's own guarantee,
//! independent of that caller ever existing.

use crate::{CheckOutcome, Claim, Run, RunContext};
use nomos_analysis::{MemoryFactStore, Reader};
use nomos_contracts::{Finding, GateCategory, RuleId};
use nomos_model::Subject_Of_Path;
use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};
use nomos_platform_std::{StdFileSystem, StdProcessLauncher};
use nomos_rules::{Check_Completeness_Mirrors, SourceFile};
use nomos_workspace::BuildVariant;
use std::cell::Cell;

fn Source(path: &str, text: &str) -> SourceFile
{
    return SourceFile::New(path, Subject_Of_Path(path), text);
}

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// This repository's own real root, for `root` -- `Run`'s dependency step runs `cargo
/// metadata` against it regardless of what `sources` a test hands in, since the dependency
/// capability is a workspace-wide fact and not a fact about any one of `sources`'s files.
/// Real on purpose, the same choice this crate's own doc comment already states for the
/// syntax half: "the real composition... which is what makes every assertion a statement
/// about the shipped seam rather than about a fixture."
fn Repository_Root() -> std::path::PathBuf
{
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    return manifest
        .parent()
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .map(std::path::PathBuf::from)
        .expect("this crate sits three levels below the workspace root");
}

/// Selected explicitly rather than left to `&[]`'s "everything" default: this repository
/// keeps its own architecture at zero findings (`COMPLETENESS_MIRROR`, `NAMING_CONVENTION`,
/// `DEPENDENCY_DIRECTION`, `DEPENDENCY_COMPLETENESS`, `UNREAD_REACHES_FINDING`), which is
/// what "a clean tree" means here, but does not keep `LINT_DIAGNOSTICS` at zero --
/// `cargo clippy`'s own pedantic-level warnings are tolerated debt this workspace's own gate
/// explicitly does not block on (`.github/workflows/gate.yml` carries no `-D warnings`), and
/// they fluctuate as concurrent sessions touch the tree. Selecting `&[]` here would make
/// this test's own pass/fail depend on the ambient lint-cleanliness of the whole real
/// repository at the moment it runs, which is not the property this test exists to prove.
fn Architectural_Rules() -> [RuleId; 5]
{
    return [
        RuleId::New(nomos_rules::COMPLETENESS_MIRROR),
        RuleId::New(nomos_rules::NAMING_CONVENTION),
        RuleId::New(nomos_rules::DEPENDENCY_DIRECTION),
        RuleId::New(nomos_rules::DEPENDENCY_COMPLETENESS),
        RuleId::New(nomos_rules::UNREAD_REACHES_FINDING),
    ];
}

#[test]
fn Test_A_Clean_Tree_Should_Be_Judged_Complete_With_No_Findings()
{
    let sources = vec![Source("a.rs", "pub fn Ok() {}\n")];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },&Architectural_Rules());

    let CheckOutcome::Judged { findings, examined, claim } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(examined, crate::Examined { files: 1, facts: 1 });
    assert_eq!(claim, Claim::Complete);
}

/// `OD-CAPABILITY-009`'s corrected fix, proven end to end: a clean `.go` source is judged
/// under `nomos_lang_go`'s own identity, the same way the `.rs` test just above is judged
/// under `nomos_lang_rust`'s -- and both stay green together, which is what proves the fix
/// does not reintroduce the regression this record's own body describes (`.rs` facts
/// resolving against `nomos_lang_go`'s stronger, unpreferenced-ranked offer instead of the
/// provider that actually wrote them).
///
/// Selects only `COMPLETENESS_MIRROR` and `NAMING_CONVENTION` -- the two rules
/// `nomos.cap.syntax.items` actually drives, and the whole of what this item fixes.
/// `UNREAD_REACHES_FINDING` has no Go provider registered for `nomos.cap.controlflow.
/// reachability` at all (`Declare_Controlflow_Capability` offers only `nomos_lang_rust`'s),
/// so selecting it here would report an honest `DependencyUnavailable` for a capability
/// this item was never asked to extend to Go, not a regression in the one it was.
#[test]
fn Test_A_Clean_Go_Source_Should_Be_Judged_Through_Its_Own_Real_Provider()
{
    let sources = vec![Source("main.go", "package main\n\nfunc One() {}\n")];
    let selected = [RuleId::New(nomos_rules::COMPLETENESS_MIRROR), RuleId::New(nomos_rules::NAMING_CONVENTION)];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },&selected);

    let CheckOutcome::Judged { findings, examined, claim } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(examined, crate::Examined { files: 1, facts: 1 });
    assert_eq!(claim, Claim::Complete);
}

/// `OD-CAPABILITY-010`'s own end-to-end proof: a real Rust source and a real Go source,
/// judged through `Run` together, over a declared correspondence whose two sides
/// genuinely disagree on fields -- not a synthetic fact built by hand, the real syntax
/// providers reading real source text.
#[test]
fn Test_A_Genuine_Cross_Language_Field_Mismatch_Should_Be_Reported()
{
    let sources = vec![
        Source(
            "wide.rs",
            "/// Corresponds to `Wide`.\npub struct Wide { pub A: u32, pub B: u32 }\n",
        ),
        Source("wide.go", "package main\n\ntype Wide struct {\n\tA int\n}\n"),
    ];
    let selected = [RuleId::New(nomos_rules::CROSS_LANGUAGE_CORRESPONDENCE)];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },&selected);

    let CheckOutcome::Judged { findings, .. } = outcome else { panic!("a tree the provider can read must be judged") };

    assert_eq!(findings.len(), 1, "{findings:?}");
    let found = findings.first().expect("asserted len 1 above");
    assert_eq!(found.applicability, nomos_contracts::Applicability::Supported);
    assert!(found.summary.contains('B'), "{}", found.summary);
}

/// The identical pair, this time agreeing on fields -- proving a real match is silent
/// rather than merely proving a real mismatch is loud.
#[test]
fn Test_A_Genuine_Cross_Language_Field_Match_Should_Report_Nothing()
{
    let sources = vec![
        Source("clean.rs", "/// Corresponds to `Clean`.\npub struct Clean { pub A: u32 }\n"),
        Source("clean.go", "package main\n\ntype Clean struct {\n\tA int\n}\n"),
    ];
    let selected = [RuleId::New(nomos_rules::CROSS_LANGUAGE_CORRESPONDENCE)];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },&selected);

    let CheckOutcome::Judged { findings, .. } = outcome else { panic!("a tree the provider can read must be judged") };

    assert!(findings.is_empty(), "{findings:?}");
}

/// A finding that can fail a build and a finding the run could not resolve a judgment
/// about are different axes -- `OD-COMPLETENESS-004`'s whole point, restated here because
/// this crate is now where both axes are actually decided. A phantom mirror is
/// `Applicability::Supported` (the rule read it and judged it false) and `GateCategory::
/// Blocking`, so it must move `nomos-cli`'s exit code and must NOT flip [`Claim`] --
/// [`Claim`] answers "did the run reach a verdict", not "did the verdict pass".
#[test]
fn Test_A_Blocking_Finding_Should_Still_Be_Judged_Complete()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
    )];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },&[]);

    let CheckOutcome::Judged { findings, claim, .. } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(!findings.is_empty(), "the phantom mirror must be reported");
    assert!(
        findings.iter().any(|finding| return finding.gate == GateCategory::Blocking),
        "{findings:?}"
    );
    assert_eq!(claim, Claim::Complete, "a blocking verdict is still a reached verdict");
}

/// A run that materializes no fact for anything it read must not be reported as judged --
/// an empty findings list here would render identically to a clean tree, which is exactly
/// the lie `nomos-cli::check`'s own vacuity guard exists to catch, one layer in.
#[test]
fn Test_A_Run_That_Materializes_No_Facts_Should_Report_NoFacts()
{
    let sources = vec![Source("broken.rs", "pub const ??? = ;")];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },&[]);

    assert!(
        matches!(outcome, CheckOutcome::NoFacts { files: 1 }),
        "a provider refusing every file must not be judged"
    );
}

/// Two sources that normalize to the same workspace member cannot both be ingested. Not a
/// case a real walk produces -- `nomos-cli::check::sources::Read_Sources` reads a real
/// directory, which cannot hand back two entries for one path -- but this crate accepts
/// `sources` from any caller, and a second adapter's own walk is not this crate's to trust.
#[test]
fn Test_Conflicting_Paths_Should_Be_Unreadable()
{
    let sources = vec![Source("a.rs", "pub fn one() {}\n"), Source("a.rs", "pub fn two() {}\n")];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },&[]);

    assert!(matches!(outcome, CheckOutcome::Unreadable), "duplicate paths must not be ingested");
}

/// The floor is the rule's, and the composition this crate assembles meets it -- the same
/// control `nomos-cli::check`'s own suite used to keep, restated here because this crate is
/// now where the composition actually lives.
#[test]
fn Test_The_Registered_Provider_Should_Satisfy_The_Rules_Floor()
{
    let sources = vec![Source("a.rs", "pub const T: &[&str] = &[];\n")];

    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },&[]);

    let CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        panic!("a tree the provider can read must be judged");
    };
    assert!(
        findings
            .iter()
            .all(|finding| return finding.applicability == nomos_contracts::Applicability::Supported),
        "the registered parser must serve nomos_rules::Syntax_Requirement: {findings:?}"
    );
    assert!(
        findings.iter().all(|finding| return finding.gate == GateCategory::Advisory),
        "{findings:?}"
    );
}

/// ---- the shipped composition consults a fact ----
///
/// The assertion `P10-FACT-BYPASS` turns on, and the reason this crate's own test suite
/// reaches past [`Run`] into its private [`crate::composition`] and [`crate::facts`]
/// pieces: the guarantee is that the rule's verdict depends on what the *store* holds, not
/// merely on which sources the rule was handed, and demonstrating that needs a store built
/// from one source list judged against a different, larger one. `Run` deliberately does not
/// expose that split -- a second adapter has no reason to ingest less than it judges -- so
/// this is where the split-composition guarantee is proven instead: internally, once, by
/// the crate that owns it.
/// The declaring/checking pair `Test_The_Composed_Crate_Should_Resolve_A_Mirror_Through_A_
/// Real_Fact` proves the split-composition guarantee over.
fn Mirror_Fixture() -> (SourceFile, SourceFile)
{
    let declaring = Source(
        "a.rs",
        "/// Mirrored by `Test_The_Real_Provider_Found_This`.\n\
         pub const T: &[&str] = &[];\n",
    );
    let checking = Source(
        "b.rs",
        "#[cfg(test)]\nmod tests\n{\n    #[test]\n    fn Test_The_Real_Provider_Found_This()\n    {\n    }\n}\n",
    );

    return (declaring, checking);
}

#[test]
fn Test_The_Composed_Crate_Should_Resolve_A_Mirror_Through_A_Real_Fact()
{
    let (declaring, checking) = Mirror_Fixture();
    let whole = vec![declaring.clone(), checking];

    let resolved = Findings_Over(&whole, &whole);
    // The store is told about the declaring file only; the rule is handed both.
    let short = Findings_Over(&[declaring], &whole);

    assert!(
        resolved.is_empty(),
        "the registered parser must find the check in b.rs: {resolved:?}"
    );
    assert!(
        short.iter().any(|finding| return finding.subject_name == "T"),
        "with b.rs's fact withheld the claim must not resolve: {short:?}"
    );
}

/// The wiring itself, proven directly: real sources flow out of the real provider over the
/// real repository root, not merely "zero findings" -- which an empty source list would
/// also produce, silently, the exact vacuity `Test_A_Clean_Tree_Should_Be_Judged_Complete_
/// With_No_Findings` cannot rule out on its own, because a `Run` that materialized nothing
/// and a `Run` that materialized a clean workspace render identically over that assertion
/// alone.
#[test]
fn Test_Materialize_Dependencies_Should_Return_Real_Workspace_Members()
{
    // What `Materialize_Dependencies` needs from a `Context` -- snapshot, variant,
    // configuration, generation -- has nothing to do with `sources`; one placeholder file
    // is enough to build a real one, the same way `Findings_Over`'s own fixtures do.
    let placeholder = [Source("placeholder.rs", "pub fn Placeholder() {}\n")];
    let registry = crate::composition::Registered().expect("fixture composition");
    let context = crate::facts::Ingested_Workspace(&placeholder, &registry, Test_Variant(), &mut None).expect("a single real file ingests");
    let mut store = MemoryFactStore::New();

    let crate::facts::DependencyMaterialization { sources, findings } =
        crate::facts::Materialize_Dependencies(&Repository_Root(), &context, &mut store, &StdProcessLauncher);

    assert!(findings.is_empty(), "a real workspace root must not report ProviderUnavailable: {findings:?}");
    assert!(
        sources.len() > 10,
        "this repository has far more than ten workspace members, so {} real sources is too \
         few to have exercised the provider: {sources:?}",
        sources.len()
    );
    assert!(
        sources.iter().any(|source| return source.path == "crates/rules/nomos-rules"),
        "expected nomos-rules among the real sources: {sources:?}"
    );
}

/// The identical claim [`Test_Materialize_Dependencies_Should_Return_Real_Workspace_Members`]
/// proves, for `nomos.cap.lint.diagnostics`: real sources flow out of the real `cargo
/// clippy` provider over the real repository root, not merely "zero findings".
#[test]
fn Test_Materialize_Lint_Should_Return_Real_Workspace_Members()
{
    let placeholder = [Source("placeholder.rs", "pub fn Placeholder() {}\n")];
    let registry = crate::composition::Registered().expect("fixture composition");
    let context = crate::facts::Ingested_Workspace(&placeholder, &registry, Test_Variant(), &mut None).expect("a single real file ingests");
    let mut store = MemoryFactStore::New();

    let crate::facts::LintMaterialization { sources, findings } =
        crate::facts::Materialize_Lint(&Repository_Root(), &context, &mut store, &StdProcessLauncher);

    assert!(findings.is_empty(), "a real workspace root must not report ProviderUnavailable: {findings:?}");
    assert!(
        sources.len() > 10,
        "this repository has far more than ten workspace members, so {} real sources is too \
         few to have exercised the provider: {sources:?}",
        sources.len()
    );
    assert!(
        sources.iter().any(|source| return source.path == "crates/rules/nomos-rules"),
        "expected nomos-rules among the real sources: {sources:?}"
    );
}

/// The identical claim [`Test_Materialize_Dependencies_Should_Return_Real_Workspace_Members`]
/// proves, for `nomos.cap.dependency.policy`: the one real fact `Materialize_Policy`
/// produces flows out of the real `cargo deny` provider over the real repository root, not
/// merely "zero findings" -- and this repository's own `deny.toml` is known to report at
/// least one real, non-`deny`-level violation (`multiple-versions = "warn"`, and this
/// workspace has duplicated dependencies today), so a genuinely absent answer would read
/// identically to an empty one without checking the payload itself.
#[test]
fn Test_Materialize_Policy_Should_Return_The_Real_Workspace_Fact()
{
    let placeholder = [Source("placeholder.rs", "pub fn Placeholder() {}\n")];
    let registry = crate::composition::Registered().expect("fixture composition");
    let context = crate::facts::Ingested_Workspace(&placeholder, &registry, Test_Variant(), &mut None).expect("a single real file ingests");
    let mut store = MemoryFactStore::New();

    let crate::facts::PolicyMaterialization { sources, findings } =
        crate::facts::Materialize_Policy(&Repository_Root(), &context, &mut store, &StdProcessLauncher);

    assert!(findings.is_empty(), "a real workspace root must not report ProviderUnavailable: {findings:?}");
    assert_eq!(sources.len(), 1, "IncrementalGranularity::WholeWorkspace materializes exactly one fact: {sources:?}");
    assert_eq!(sources.first().expect("asserted len 1 above").path, "workspace");
}

/// A launcher that counts how many times it was asked to run something, and refuses every
/// one -- proving `OD-GATE-017`'s claim that a deselected rule's own materialization does
/// not run at all, which a real invocation's findings cannot distinguish from "ran and found
/// nothing" on their own.
struct CountingLauncher
{
    calls: Cell<usize>,
}

impl CountingLauncher
{
    fn New() -> Self
    {
        return Self { calls: Cell::new(0) };
    }

    fn Count(&self) -> usize
    {
        return self.calls.get();
    }
}

impl ProcessLauncher for CountingLauncher
{
    fn Run(&self, _command: &Command) -> Result<ProcessOutput, String>
    {
        self.calls.set(self.calls.get().saturating_add(1));
        return Ok(ProcessOutput { outcome: ExitOutcome::Exited { code: 1 }, stdout: String::new(), stderr: String::new() });
    }
}

/// `Run`'s `selected` parameter must actually gate computation, not merely gate what a
/// disposition later discards -- `OD-GATE-017`'s own point. Findings cannot prove this: a
/// healthy repository's `cargo metadata` call raises no finding on success, so "deselected"
/// and "selected but clean" would render identically over `findings` alone. Counting real
/// launches is the only distinguishing evidence.
#[test]
fn Test_A_Deselected_Dependency_Rule_Should_Not_Launch_Cargo_Metadata()
{
    let sources = vec![Source("a.rs", "pub fn Ok() {}\n")];

    let unselected = CountingLauncher::New();
    let _ = Run(
        &sources,
        RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &unselected, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },
        &[RuleId::New(nomos_rules::COMPLETENESS_MIRROR)],
    );
    assert_eq!(unselected.Count(), 0, "dependency-direction was not selected, so cargo metadata must not run");

    let selected = CountingLauncher::New();
    let _ = Run(
        &sources,
        RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &selected, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },
        &[RuleId::New(nomos_rules::DEPENDENCY_DIRECTION)],
    );
    assert_eq!(selected.Count(), 1, "dependency-direction was selected, so cargo metadata must run exactly once");
}

/// The identical claim [`Test_A_Deselected_Dependency_Rule_Should_Not_Launch_Cargo_Metadata`]
/// proves, for `LINT_DIAGNOSTICS` and `cargo clippy`.
#[test]
fn Test_A_Deselected_Lint_Rule_Should_Not_Launch_Cargo_Clippy()
{
    let sources = vec![Source("a.rs", "pub fn Ok() {}\n")];

    let unselected = CountingLauncher::New();
    let _ = Run(
        &sources,
        RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &unselected, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },
        &[RuleId::New(nomos_rules::COMPLETENESS_MIRROR)],
    );
    assert_eq!(unselected.Count(), 0, "lint-diagnostics was not selected, so cargo clippy must not run");

    let selected = CountingLauncher::New();
    let _ = Run(
        &sources,
        RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &selected, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },
        &[RuleId::New(nomos_rules::LINT_DIAGNOSTICS)],
    );
    assert_eq!(selected.Count(), 1, "lint-diagnostics was selected, so cargo clippy must run exactly once");
}

/// The identical claim [`Test_A_Deselected_Dependency_Rule_Should_Not_Launch_Cargo_Metadata`]
/// proves, for `DEPENDENCY_POLICY` and `cargo deny`.
#[test]
fn Test_A_Deselected_Dependency_Policy_Rule_Should_Not_Launch_Cargo_Deny()
{
    let sources = vec![Source("a.rs", "pub fn Ok() {}\n")];

    let unselected = CountingLauncher::New();
    let _ = Run(
        &sources,
        RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &unselected, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },
        &[RuleId::New(nomos_rules::COMPLETENESS_MIRROR)],
    );
    assert_eq!(unselected.Count(), 0, "dependency-policy was not selected, so cargo deny must not run");

    let selected = CountingLauncher::New();
    let _ = Run(
        &sources,
        RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &selected, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },
        &[RuleId::New(nomos_rules::DEPENDENCY_POLICY)],
    );
    assert_eq!(selected.Count(), 1, "dependency-policy was selected, so cargo deny must run exactly once");
}

/// Ingests `ingested` into a real fact store and judges `judged` over it -- the split
/// [`crate::run_context::Run`] does not offer, assembled here from the crate's own private pieces.
///
/// `judged` is run through [`crate::run_context::Recognized_Sources`] before the rule ever sees it, the
/// identical enrichment `Run` gives every real caller: `Check_Completeness_Mirrors` narrows
/// its own `Require` call by `SourceFile::preferred_syntax_provider`, and a fixture built by
/// hand needs that field populated the same way a real walk's sources would be, or `.rs`
/// resolves against `nomos_lang_go`'s stronger, unpreferenced-ranked offer instead of the
/// provider that actually wrote the fact.
fn Findings_Over(ingested: &[SourceFile], judged: &[SourceFile]) -> Vec<Finding>
{
    let registry = crate::composition::Registered().expect("the fixture composition is this crate's own");
    let context = crate::facts::Ingested_Workspace(ingested, &registry, Test_Variant(), &mut None).expect("the fixture is a valid tree");
    let mut store = MemoryFactStore::New();
    let _written = crate::facts::Materialize_Syntax(ingested, &context, &mut store);

    let judged = crate::run_context::Recognized_Sources(judged);
    let mut reader = Reader::On(&store, &registry, context);
    return Check_Completeness_Mirrors(&judged, &mut reader);
}

/// `OD-ANALYSIS-009`'s first real increment, exercised end to end: `crates/substrate/
/// nomos-analysis/tests/recomputation_equivalence.rs` already proves, over synthetic
/// `FactKey`s built by hand, that `MemoryFactStore::Invalidate` and `Materialize` agree with
/// a clean rebuild. What that file cannot prove is that this crate's own real composition --
/// the real syntax provider, the real `Workspace`, `Run`'s real ingestion and judging -- ever
/// asks a store and a workspace to survive across two calls at all; before `Run` accepted
/// either through `RunContext`, nothing in this workspace ever did. This is that second real
/// caller: the same `workspace` and `store` carried across a call that edits one file and
/// leaves another alone, checked against an independent third call that recomputes the
/// post-edit tree from nothing.
///
/// This does not prove `Run` skips recomputing anything for the untouched file --
/// `Materialize_Syntax` still re-derives every source's fact on every call regardless of
/// whether `store` already holds a live one, so `IncrementalResult` here costs the same work
/// `CleanRecomputation` does. What it proves is the precondition that gap's fix would need:
/// carrying a workspace and a store across a real edit is *safe* -- reusing them agrees with
/// throwing them away and starting over, on both the claim and the findings, and the
/// generation the reused workspace reports genuinely advances rather than repeating itself.
/// A caller could not have relied on either fact before this increment, because no caller
/// had ever reused either object.
#[test]
fn Test_A_Store_And_Workspace_Reused_Across_An_Edit_Agrees_With_A_Clean_Recomputation()
{
    let unedited = Source("a.rs", "pub fn Ok() {}\n");
    let edited = Source("a.rs", "pub fn Ok() {}\npub fn Also_Ok() {}\n");
    let untouched = Source("b.rs", "pub fn Untouched() {}\n");
    let selected = [RuleId::New(nomos_rules::NAMING_CONVENTION)];

    let mut workspace = None;
    let mut store = MemoryFactStore::New();
    let first = Run(
        &[unedited, untouched.clone()],
        RunContext {
            variant: Test_Variant(),
            root: &Repository_Root(),
            launcher: &StdProcessLauncher,
            filesystem: &StdFileSystem,
            workspace: &mut workspace,
            store: &mut store,
        },
        &selected,
    );
    assert!(matches!(first, CheckOutcome::Judged { .. }), "the first call over a readable tree must be judged");
    let generation_after_first = workspace.as_ref().expect("Run must leave a workspace behind").Generation();

    let incremental = Run(
        &[edited.clone(), untouched.clone()],
        RunContext {
            variant: Test_Variant(),
            root: &Repository_Root(),
            launcher: &StdProcessLauncher,
            filesystem: &StdFileSystem,
            workspace: &mut workspace,
            store: &mut store,
        },
        &selected,
    );
    let generation_after_second = workspace.as_ref().expect("Run must leave a workspace behind").Generation();
    assert!(
        generation_after_second > generation_after_first,
        "a real edit reusing the same workspace must advance its generation, not repeat it: \
         {generation_after_first} then {generation_after_second}"
    );

    let mut clean_workspace = None;
    let mut clean_store = MemoryFactStore::New();
    let clean = Run(
        &[edited, untouched],
        RunContext {
            variant: Test_Variant(),
            root: &Repository_Root(),
            launcher: &StdProcessLauncher,
            filesystem: &StdFileSystem,
            workspace: &mut clean_workspace,
            store: &mut clean_store,
        },
        &selected,
    );

    let CheckOutcome::Judged { findings: incremental_findings, claim: incremental_claim, .. } = incremental
    else
    {
        panic!("the reused workspace and store call over a readable tree must be judged");
    };
    let CheckOutcome::Judged { findings: clean_findings, claim: clean_claim, .. } = clean
    else
    {
        panic!("the fresh workspace and store call over a readable tree must be judged");
    };

    assert_eq!(
        incremental_claim, clean_claim,
        "a workspace and store reused across an edit must reach the same claim a clean recomputation reaches"
    );
    assert_eq!(
        incremental_findings, clean_findings,
        "a workspace and store reused across an edit must report the same findings a clean recomputation reports"
    );
}

/// `RunContext`'s own `filesystem` reaching a real repository-declared policy fact, not just
/// compiling: without a real `standards.json` under `root`, `NAMING_CONVENTION` has always
/// judged every function name against its own hardcoded `UpperSnake` default. This proves the
/// same function, judged twice, disagrees depending only on what `root`'s own real
/// `standards.json` (read through a real `StdFileSystem`) declares -- the naming-policy
/// materialization this item wires actually reaches `nomos_rules::Resolve_Case`, not merely a
/// port that type-checks.
#[test]
fn Test_Run_Should_Honor_A_Real_Standards_Json_Naming_Override()
{
    let sources = vec![Source("a.rs", "pub fn lower_snake_name() {}\n")];
    let selected = [RuleId::New(nomos_rules::NAMING_CONVENTION)];

    let unconfigured = Scratch_Directory("naming-unconfigured");
    let unconfigured_outcome = Run(
        &sources,
        RunContext { variant: Test_Variant(), root: &unconfigured, launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },
        &selected,
    );
    let CheckOutcome::Judged { findings: unconfigured_findings, .. } = unconfigured_outcome
    else
    {
        panic!("a tree with no standards.json must still be judged, against the hardcoded default");
    };
    assert_eq!(
        unconfigured_findings.len(),
        1,
        "an all-lowercase function name violates the hardcoded UpperSnake default when nothing overrides it: {unconfigured_findings:?}"
    );

    let overridden = Scratch_Directory("naming-overridden");
    std::fs::write(overridden.join("standards.json"), r#"{"naming":{"function":"lower-snake"}}"#).expect("a scratch standards.json");
    let overridden_outcome = Run(
        &sources,
        RunContext { variant: Test_Variant(), root: &overridden, launcher: &StdProcessLauncher, filesystem: &StdFileSystem, workspace: &mut None, store: &mut MemoryFactStore::New() },
        &selected,
    );
    let CheckOutcome::Judged { findings: overridden_findings, .. } = overridden_outcome
    else
    {
        panic!("a tree with a real standards.json must still be judged")
    };
    assert!(
        overridden_findings.is_empty(),
        "a repository declaring naming.function = \"lower-snake\" must accept a lower_snake function name: {overridden_findings:?}"
    );
}

/// A fresh, empty directory under the OS temp root, unique per test name and process --
/// `nomos-cli::work`'s own `Scratch_Directory` fixture shape, needed here for the same reason:
/// `StdFileSystem` reads real bytes from a real path, so proving a real override changes real
/// behavior needs a real file on disk rather than a fixture handed in by value.
fn Scratch_Directory(name: &str) -> std::path::PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-check-orchestration-test-{name}-{}", std::process::id()));
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a scratch directory");
    return root;
}

