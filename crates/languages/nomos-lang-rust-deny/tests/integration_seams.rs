//! The real crate-to-crate seams `nomos-lang-rust-deny` reaches across, driven only
//! through this crate's own public API -- the same view a real consumer has.
//!
//! `nomos-lang-rust-deny` depends on six crates in its own source -- `nomos_contracts`,
//! `nomos_model`, `nomos_capability`, `nomos_analysis`, `nomos_cap_dependency_policy` and
//! `nomos_platform` -- plus `nomos_platform_std` as the real launcher a composition root
//! supplies through the `nomos_platform::ProcessLauncher` port. Each test below exercises
//! the actual call this crate makes into one of them.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SnapshotId};
use nomos_lang_rust_deny::{
    Declared_Guarantee, Discover_Workspace, FactContext, Materialize_Workspace, PolicyFact, Provider_Offer,
};
use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};
use nomos_platform_std::StdEnvironment;
use std::path::{Path, PathBuf};

/// A launcher that hands `Discover_Workspace` a fixed stderr stream instead of running a
/// real `cargo deny` -- the same substitution this crate's own module doc names as the one
/// place a caller supplies a real subprocess, reproduced here because the crate's own
/// equivalent (`reading::tests::FakeLauncher`) is private to its own test module.
struct FakeLauncher
{
    stderr: String,
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for FakeLauncher
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProcessLauncher for FakeLauncher
{
    fn Run(&self, _command: &Command) -> Result<ProcessOutput, String>
    {
        return Ok(ProcessOutput {
            outcome: ExitOutcome::Exited { code: 0 },
            stdout: String::new(),
            stderr: self.stderr.clone(),
        });
    }
}

/// One real violation shape, as `cargo deny`'s own `--format json` stream reports it --
/// kept as a table even though every test below shares the one case, so a second shape
/// discovered later has a natural place to land rather than a second hardcoded literal.
fn Sample_Violation_Cases() -> Vec<(&'static str, &'static str, &'static str)>
{
    return vec![("duplicate", "warning", "multiple versions of a crate are present")];
}

/// `cases`, one `cargo deny` diagnostic JSON object per line -- the exact shape
/// `reading::Violation_Of` reads, reproduced here because that reader is private to the
/// crate's own module.
fn Diagnostics_Of(cases: &[(&str, &str, &str)]) -> String
{
    return cases
        .iter()
        .map(|(code, severity, message)| {
            return serde_json::json!({
                "type": "diagnostic",
                "fields": {
                    "code": code,
                    "severity": severity,
                    "message": message,
                    "labels": [],
                    "graphs": []
                }
            })
            .to_string();
        })
        .collect::<Vec<_>>()
        .join("\n");
}

/// The `{"type":"summary"}` line a finished `cargo deny check bans licenses sources` run
/// always writes, naming every check this provider asks for.
///
/// `P81` made that line the difference between a run whose silence is a clean result and
/// one whose silence is a failure nobody could read. A fixture standing for a completed run
/// has to be shaped like one; without it these seams would be asserting about the refusal
/// path while claiming to test the parse.
fn Completed_Summary() -> String
{
    return serde_json::json!({
        "type": "summary",
        "fields": {
            "bans": { "errors": 0, "helps": 0, "notes": 0, "warnings": 0 },
            "licenses": { "errors": 0, "helps": 0, "notes": 0, "warnings": 0 },
            "sources": { "errors": 0, "helps": 0, "notes": 0, "warnings": 0 }
        }
    })
    .to_string();
}

/// The whole stderr stream a finished run writes: the diagnostics, then the summary.
fn Diagnostics_Stderr(cases: &[(&str, &str, &str)]) -> String
{
    return format!("{}\n{}", Diagnostics_Of(cases), Completed_Summary());
}

/// Fill bytes distinct enough that the three digests below differ from one another; each
/// value carries no meaning beyond "not equal to the others".
const VARIANT_DIGEST_FILL: u8 = 2;
const CONFIGURATION_DIGEST_FILL: u8 = 3;

fn Context() -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([VARIANT_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_DIGEST_FILL; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}

/// Runs `Materialize_Workspace` over the fake launcher's own canned violation -- the one
/// path every test below that needs a real [`PolicyFact`] shares, rather than
/// `fact_context::tests`'s own real, whole-repository `cargo deny` invocation.
///
/// Over a real [`ScratchDirectory`] (defined further down, alongside the one real-process
/// test that also needs it) rather than `Path::new(".")`: `Discover_Workspace` now refuses
/// before ever reaching a launcher -- fake or real -- unless `root` has its own `deny.toml`
/// (`P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT`), and `.` here would resolve to this
/// crate's own manifest directory, which has none. `name` disambiguates each caller's own
/// scratch directory, since `cargo test` runs this file's tests concurrently.
fn Materialize_With(cases: &[(&str, &str, &str)], name: &str) -> PolicyFact
{
    let scratch = ScratchDirectory::New(name);
    let launcher = FakeLauncher { stderr: Diagnostics_Stderr(cases) };

    return Materialize_Workspace(scratch.Path(), Context(), &launcher, &StdEnvironment)
        .expect("the fake launcher writes a real stderr stream Discover_Workspace can read");
}

/// `nomos_platform`: `Discover_Workspace` is generic over `nomos_platform::ProcessLauncher`
/// -- a caller-supplied implementation of that port is exactly what this reads a real
/// diagnostic shape through, previously exercised nowhere in this crate's own `tests/`.
#[test]
fn Test_Discover_Workspace_Should_Read_A_Real_Diagnostic_Shape_Through_The_Process_Launcher_Port()
{
    let scratch = ScratchDirectory::New("read-diagnostic-shape");
    let cases = Sample_Violation_Cases();
    let launcher = FakeLauncher { stderr: Diagnostics_Stderr(&cases) };

    let violations = Discover_Workspace(scratch.Path(), &launcher, &StdEnvironment)
        .expect("the fake launcher writes a real stderr stream Discover_Workspace can read");

    let (code, severity_label, message) = cases.first().expect("one sample case");
    let expected_severity = nomos_cap_dependency_policy::PolicySeverity::From_Label(severity_label)
        .expect("this table's own severity label is one PolicySeverity recognizes");
    let violation = violations.first().expect("one violation");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(&violation.code, code);
    assert_eq!(violation.severity, expected_severity);
    assert_eq!(&violation.message, message);
}

/// `nomos_model`: the fact this crate materializes is filed under `nomos_model::
/// Subject_Of_Path`'s own whole-workspace subject -- the same construction
/// `Materialize_Workspace` performs internally through its own private `Compute_Fact_Key`,
/// checked here through the public API a real consumer has, thinned from the inline
/// `fact_context::tests::Test_Discover_Workspace_And_Materialize_Workspace_Should_Find_
/// Every_Real_Policy_Violation` so it no longer needs a real `cargo deny` process to prove.
#[test]
fn Test_The_Materialized_Facts_Subject_Should_Be_Nomos_Models_Subject_Of_Path()
{
    let PolicyFact { subject, .. } = Materialize_With(&Sample_Violation_Cases(), "facts-subject");

    assert_eq!(subject, nomos_model::Subject_Of_Path(""));
}

/// `nomos_capability`: this crate's own offer is accepted under `nomos_cap_dependency_
/// policy`'s own contract through `nomos_capability::Registry` -- the composition-time
/// check every real caller relies on, previously exercised nowhere in this crate's own
/// `tests/`.
#[test]
fn Test_Provider_Offer_Should_Be_Accepted_By_Nomos_Capabilitys_Own_Registry()
{
    use nomos_capability::Registry;

    let mut registry = Registry::New();
    registry
        .Declare(nomos_cap_dependency_policy::Capability_Contract())
        .expect("the contract is the first declaration in a fresh registry");

    assert_eq!(registry.Offer(Provider_Offer()), Ok(()));
}

/// `nomos_analysis`: the fact this crate materializes is a real `nomos_analysis::
/// MaterializedFact` that `nomos_analysis`'s own store accepts, and reads back under the
/// key this crate computed -- the happy-path flow across the boundary, previously
/// exercised nowhere in this crate's own `tests/`.
#[test]
fn Test_A_Materialized_Fact_Should_Be_Accepted_And_Read_Back_By_Nomos_Analysiss_Own_Store()
{
    use nomos_analysis::{FactStore, MemoryFactStore};

    let PolicyFact { fact, .. } = Materialize_With(&Sample_Violation_Cases(), "analysis-store");
    let key = fact.Key().clone();
    let mut store = MemoryFactStore::New();

    store
        .Materialize(fact, &[])
        .expect("nomos_analysis accepts this crate's own fact shape without complaint");

    let current = store
        .Current(&key.At(GenerationId::INITIAL), GenerationId::INITIAL)
        .expect("nomos_analysis can address the fact this crate wrote by the key this crate computed");
    assert_eq!(current.guarantee, Declared_Guarantee());
}

/// `nomos_cap_dependency_policy` and, through it, `nomos_contracts`: this crate's own
/// declared guarantee satisfies the capability's ceiling via `nomos_contracts::Guarantee::
/// Satisfies` -- the exact check `nomos_capability::Registry::Offer` runs at composition
/// time, thinned from the inline `guarantee::tests::Test_The_Ceiling_Should_Satisfy_The_
/// Declared_Guarantee` so it is checked through the public API a real consumer has instead
/// of `super::*`'s crate-internal access.
#[test]
fn Test_This_Crates_Declared_Guarantee_Should_Satisfy_The_Capabilitys_Own_Ceiling()
{
    use nomos_cap_dependency_policy::Ceiling;

    assert!(Ceiling().Satisfies(&Declared_Guarantee()));
}

/// A scratch directory with no `Cargo.toml` but its own minimal `deny.toml`, removed when
/// the test ends -- shared by every test above that needs a real root `Discover_Workspace`
/// will accept, fake launcher or real, plus the one real-process test below it was
/// originally written for.
///
/// The `deny.toml` is required, not incidental: `Required_Deny_Config` now refuses before
/// ever launching `cargo deny` when `root` has none of its own
/// (`P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT`), regardless of which launcher would
/// have answered -- `Path::new(".")` (this crate's own manifest directory, which has no
/// `deny.toml`) stopped being a valid stand-in root the moment that check landed.
struct ScratchDirectory
{
    root: PathBuf,
}

impl ScratchDirectory
{
    fn New(name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-lang-rust-deny-scratch-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory outside any cargo workspace");
        std::fs::write(root.join("deny.toml"), "[bans]\nmultiple-versions = \"allow\"\n").expect("writing the scratch deny.toml");

        return Self { root };
    }

    fn Path(&self) -> &Path
    {
        return &self.root;
    }
}

impl Drop for ScratchDirectory
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

/// `nomos_platform_std`: `Discover_Workspace` runs a real process through this crate's real
/// `StdProcessLauncher`, proportionately -- a real `cargo deny` invocation outside any
/// cargo workspace, which returns in well under a second because it fails to find a
/// manifest before resolving anything, rather than duplicating `fact_context::tests`'s own
/// real, whole-repository invocation.
///
/// # This asserted the opposite until `P81`
///
/// It used to `expect` success here and assert the violation list was empty, on the reading
/// that a run outside a workspace "still answers -- with no diagnostic lines to parse,
/// rather than a launcher failure". Measured with the real tool, that invocation exits 1
/// and writes exactly one line: `{"type":"log"}` at level `ERROR` saying the directory
/// doesn't contain a `Cargo.toml` file. No summary, no diagnostics, nothing judged. It is a
/// failure that resolved no dependency graph at all, and the old assertion enshrined
/// reporting it as a clean dependency-policy result -- the exact false clean `P81` closes,
/// written down as intended behaviour by a test that passed.
///
/// The distance between "answers with nothing to say" and "could not look" is the whole
/// property, and it is worth one real process to hold it.
#[test]
fn Test_A_Real_Run_That_Resolved_No_Workspace_Should_Be_Refused_Rather_Than_Reported_Clean()
{
    use nomos_platform_std::{StdEnvironment, StdProcessLauncher};

    let scratch = ScratchDirectory::New("no-manifest");

    let error = Discover_Workspace(scratch.Path(), &StdProcessLauncher, &StdEnvironment).expect_err(
        "a real cargo deny that could not resolve a workspace at all has not judged this \
         directory's dependencies, and must be refused rather than reported clean",
    );

    assert!(
        error.reason.contains("no summary line"),
        "the refusal must rest on cargo deny's own missing summary, not on a guess: {}",
        error.reason
    );
    assert!(
        error.reason.contains("Cargo.toml"),
        "and it must quote what cargo deny itself said went wrong: {}",
        error.reason
    );
}
