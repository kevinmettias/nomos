//! The subprocess-backed capabilities, exercised over this repository's own real root: what
//! `cargo metadata`, `cargo clippy` and `cargo deny` actually hand back, and the launch a
//! rule the caller did not select must never pay for.

use nomos_analysis::MemoryFactStore;
use nomos_contracts::{Finding, RuleId};
use nomos_platform::{Command, DeterminismStrength, ExitOutcome, ProcessLauncher, ProcessOutput};
use nomos_platform::{ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
use nomos_rules::SourceFile;
use std::cell::Cell;

use crate::{CheckOutcome, Run, RunContext};

use super::{Repository_Root, Source, SourceText, Test_Variant};

/// This workspace has far more members than this, so a real materialization that returned
/// fewer sources than this never reached the provider at all.
const REAL_WORKSPACE_MEMBER_FLOOR: usize = 10;

/// What `Materialize_Dependencies`, `Materialize_Lint` and `Materialize_Policy` need from a
/// `Context` -- snapshot, variant, configuration, generation -- has nothing to do with the
/// sources judged, so one placeholder file is enough to build a real one.
fn Ingested_Placeholder() -> nomos_analysis::Context
{
    let placeholder = [Source("placeholder.rs", SourceText("pub fn Placeholder() {}\n"))];
    let registry = crate::composition::Registered().expect("fixture composition");
    return crate::facts::Ingested_Workspace(&placeholder, &registry, Test_Variant(), &mut None)
        .expect("a single real file ingests");
}

/// The real sources a subprocess-backed capability materialized over this repository's own
/// root, with its own findings asserted empty -- a real root must not report a provider
/// unavailable, which is what makes the sources below the provider's own output rather than
/// a fixture's.
fn Materialized_Over_This_Repository<Materialize>(materialize: Materialize) -> Vec<SourceFile>
where
    Materialize: FnOnce(&mut MemoryFactStore) -> (Vec<SourceFile>, Vec<Finding>),
{
    let mut store = MemoryFactStore::New();
    let (sources, findings) = materialize(&mut store);
    assert!(
        findings.is_empty(),
        "a real workspace root must not report ProviderUnavailable: {findings:?}"
    );
    return sources;
}

/// Asserts a real subprocess-backed capability returned the workspace's real member sources,
/// not merely "zero findings" -- which an empty source list would also produce, silently.
fn Assert_Real_Workspace_Sources(sources: &[SourceFile])
{
    assert!(
        sources.len() > REAL_WORKSPACE_MEMBER_FLOOR,
        "this repository has far more than {REAL_WORKSPACE_MEMBER_FLOOR} workspace members, so {} real sources is too few to have exercised the provider: {sources:?}",
        sources.len()
    );
    assert!(
        sources.iter().any(|source| return source.path == "crates/rules/nomos-rules"),
        "expected nomos-rules among the real sources: {sources:?}"
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
    let sources = Materialized_Over_This_Repository(|store| {
        let crate::facts::DependencyMaterialization { sources, findings } =
            crate::facts::Materialize_Dependencies(&Repository_Root(), &Ingested_Placeholder(), store, crate::facts::Subprocess { launcher: &StdProcessLauncher, environment: &StdEnvironment });
        return (sources, findings);
    });

    Assert_Real_Workspace_Sources(&sources);
}

/// The identical claim [`Test_Materialize_Dependencies_Should_Return_Real_Workspace_Members`]
/// proves, for `nomos.cap.lint.diagnostics`: real sources flow out of the real `cargo
/// clippy` provider over the real repository root, not merely "zero findings".
#[test]
fn Test_Materialize_Lint_Should_Return_Real_Workspace_Members()
{
    let sources = Materialized_Over_This_Repository(|store| {
        let crate::facts::LintMaterialization { sources, findings } =
            crate::facts::Materialize_Lint(&Repository_Root(), &Ingested_Placeholder(), store, crate::facts::Subprocess { launcher: &StdProcessLauncher, environment: &StdEnvironment });
        return (sources, findings);
    });

    Assert_Real_Workspace_Sources(&sources);
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
    let sources = Materialized_Over_This_Repository(|store| {
        let crate::facts::PolicyMaterialization { sources, findings } =
            crate::facts::Materialize_Policy(&Repository_Root(), &Ingested_Placeholder(), store, crate::facts::Subprocess { launcher: &StdProcessLauncher, environment: &StdEnvironment });
        return (sources, findings);
    });

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

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for CountingLauncher
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProcessLauncher for CountingLauncher
{
    fn Run(&self, _command: &Command) -> Result<ProcessOutput, String>
    {
        self.calls.set(self.calls.get().saturating_add(1));
        return Ok(ProcessOutput { outcome: ExitOutcome::Exited { code: 1 }, stdout: String::new(), stderr: String::new() });
    }
}

/// Asserts that a rule needing a subprocess launches it exactly `expected` times, with a rule
/// that reads only sources as the control that must launch nothing at all.
fn Assert_Launches_Exactly(selected: &RuleId, expected: usize)
{
    assert_eq!(Launches_Of(&[RuleId::New(nomos_rules::COMPLETENESS_MIRROR)]), 0, "a rule reading only sources must not reach the launcher");
    assert_eq!(Launches_Of(std::slice::from_ref(selected)), expected, "the selected rule's own materialization must launch exactly {expected} process(es)");
}

/// How many times `Run` asked a counting launcher to run a process while judging one clean
/// fixture source under `selected`.
fn Launches_Of(selected: &[RuleId]) -> usize
{
    let launcher = CountingLauncher::New();
    let sources = [Source("a.rs", SourceText("pub fn Ok() {}\n"))];
    let outcome = Run(&sources, RunContext { variant: Test_Variant(), root: &Repository_Root(), launcher: &launcher, filesystem: &StdFileSystem, environment: &StdEnvironment, workspace: &mut None, store: &mut MemoryFactStore::New() }, selected);
    assert!(!matches!(outcome, CheckOutcome::Unreadable), "one real, well-formed source cannot be an unreadable tree");

    return launcher.Count();
}

/// `Run`'s `selected` parameter must actually gate computation, not merely gate what a
/// disposition later discards -- `OD-GATE-017`'s own point. Findings cannot prove this: a
/// healthy repository's `cargo metadata` call raises no finding on success, so "deselected"
/// and "selected but clean" would render identically over `findings` alone. Counting real
/// launches is the only distinguishing evidence.
#[test]
fn Test_A_Deselected_Dependency_Rule_Should_Not_Launch_Cargo_Metadata()
{
    Assert_Launches_Exactly(&RuleId::New(nomos_rules::DEPENDENCY_DIRECTION), 1);
}

/// The identical claim [`Test_A_Deselected_Dependency_Rule_Should_Not_Launch_Cargo_Metadata`]
/// proves, for `LINT_DIAGNOSTICS` and `cargo clippy`.
#[test]
fn Test_A_Deselected_Lint_Rule_Should_Not_Launch_Cargo_Clippy()
{
    Assert_Launches_Exactly(&RuleId::New(nomos_rules::LINT_DIAGNOSTICS), 1);
}

/// The identical claim [`Test_A_Deselected_Dependency_Rule_Should_Not_Launch_Cargo_Metadata`]
/// proves, for `DEPENDENCY_POLICY` and `cargo deny`.
#[test]
fn Test_A_Deselected_Dependency_Policy_Rule_Should_Not_Launch_Cargo_Deny()
{
    Assert_Launches_Exactly(&RuleId::New(nomos_rules::DEPENDENCY_POLICY), 1);
}
