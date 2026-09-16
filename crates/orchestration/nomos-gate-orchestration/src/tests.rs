//! What this crate promises today: a real rule registry, reported back whole, and a real
//! disposition reduction over an already-judged list of findings.
//!
//! Split by the question each group answers, because one file holding all of them passed the
//! workspace's own 500-line review trigger: `registry` what a plan reports, `disposition` what
//! one finding does to a verdict, `run` what a whole run does with a policy, `baseline` what a
//! declared quantity does to a tolerated scope, `explain` what one named finding answers,
//! `coverage` what a floor does to an incomplete run, `declared_policy` what a file adds to a
//! command, and `narrowing` what a scope does to a rule answering a cross-file question. Every
//! fixture two of those share lives here rather than in either one.

mod baseline;
mod coverage;
mod declared_policy;
mod disposition;
mod explain;
mod narrowing;
mod registry;
mod run;

use crate::{
    AdoptionPolicy, BaselineAllowance, BaselineDebt, BaselinePolicy, Explain_Gate, FindingQuery, GateCommand, GateEnvironment,
    GateExplainResult, GateRunOutcome, GateRunResult, RuleCalibration, Run_Gate, Suppression, SuppressionDisposition, SuppressionPolicy,
};
use nomos_contracts::{Digest128, Finding, RunId};
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::PathBuf;

/// One `a.rs` no shipped rule reports anything about, so a run over it is clean.
///
/// A constant rather than the same literal at each call site, so a fixture cannot drift into
/// silently judging a source no rule says anything about.
const CLEAN_SOURCE: &str = "pub fn Ok()\n{\n}\n";

/// The one `a.rs` source `Check_Completeness_Mirrors` reports a blocking finding for: a constant
/// whose doc claims a mirror named `Test_Nowhere`, which no source any fixture here declares.
///
/// A constant for the reason [`CLEAN_SOURCE`] is one, and because this is the premise every
/// blocking-finding test rests on: a fixture that drifted off it would still pass its own
/// assertions while proving nothing.
const MIRRORED_SOURCE: &str = "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n";

/// A fixture's source path, as a type distinct from the text at it.
///
/// [`Source_File`] takes two adjacent string positions, which this crate's own gate reported: a caller
/// can transpose them and the compiler will not object. Two named types make the transposition a
/// type error, and cost one line each.
#[derive(Clone, Copy)]
struct SourcePath<'a>(&'a str);

/// The text of a fixture's source file, distinct from the path it sits at.
#[derive(Clone, Copy)]
struct SourceText<'a>(&'a str);

/// A source at `path` carrying `text`, with the subject a real walk would give it.
fn Source_File(path: SourcePath<'_>, text: SourceText<'_>) -> SourceFile
{
    return SourceFile::New(path.0, nomos_model::Subject_Of_Path(path.0), text.0.to_owned());
}

/// [`CLEAN_SOURCE`], as `path`'s own file.
fn Clean_Source(path: SourcePath<'_>) -> SourceFile
{
    return Source_File(path, SourceText(CLEAN_SOURCE));
}

/// [`MIRRORED_SOURCE`], as `path`'s own file.
fn Mirrored_Source(path: SourcePath<'_>) -> SourceFile
{
    return Source_File(path, SourceText(MIRRORED_SOURCE));
}

/// The build variant every fixture judges as.
fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// A fixed `RunId` for tests that judge a run's findings and disposition, not its identity.
/// `Fresh_Run_Id` has its own tests for that.
fn Test_Run_Id() -> RunId
{
    return RunId::From_Digest(Digest128::From_Bytes([0; Digest128::BYTE_LENGTH]));
}

/// The variant, launcher, filesystem, environment and moment every fixture judges under.
///
/// One place rather than the same five-field literal at every call site. The launcher and the
/// filesystem are the platform a run is judged *through* rather than a fixture's own choice, so a
/// fixture that spelled them out again would be restating a composition-root decision and would be
/// the one place a test could silently judge under a different one than the test beside it.
fn Gate_Platform() -> GateEnvironment<'static, StdProcessLauncher, StdFileSystem, StdEnvironment>
{
    return GateEnvironment {
        variant: Test_Variant(),
        launcher: &StdProcessLauncher,
        filesystem: &StdFileSystem,
        environment: &StdEnvironment,
        now: nomos_platform::Timestamp::From_Unix_Seconds(0),
    };
}

/// `sources` judged by `command`, under every default a fixture works with.
fn Ran_Over(sources: Vec<SourceFile>, command: &GateCommand) -> GateRunResult
{
    return Run_Gate(Some(sources), Gate_Platform(), command, Test_Run_Id());
}

/// A root nobody walked, judged identically.
fn Ran_Unwalked(command: &GateCommand) -> GateRunResult
{
    return Run_Gate(None, Gate_Platform(), command, Test_Run_Id());
}

/// `query` answered over `sources`, under every default a fixture works with.
fn Explained_Over(sources: Vec<SourceFile>, command: &GateCommand, query: &FindingQuery) -> GateExplainResult
{
    return Explain_Gate(Some(sources), Gate_Platform(), command, query);
}

/// A [`GateCommand`] selecting everything, over the current directory.
fn Command() -> GateCommand
{
    return GateCommand { root: PathBuf::from("."), ..Default::default() };
}

/// [`Command`] over `root`, everything else select-everything.
fn Command_At(root: PathBuf) -> GateCommand
{
    return GateCommand { root, ..Default::default() };
}

/// This repository's own real root -- [`Run_Gate`]'s dependency step, through
/// `nomos_check_orchestration::Run`, runs `cargo metadata` against it regardless of what
/// sources a test hands in. Real on purpose, the same choice `nomos-check-orchestration`'s
/// own tests already make.
fn Repository_Root() -> PathBuf
{
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    return manifest
        .parent()
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .map(PathBuf::from)
        .expect("this crate sits three levels below the workspace root");
}

/// [`Command_At`] `root`, with `suppression` as the whole suppression policy and every rule
/// still selected -- the shape both a `run` and an `explain` fixture build once they have a
/// real finding to suppress.
fn Command_With_Suppression(root: PathBuf, suppression: Suppression) -> GateCommand
{
    return GateCommand {
        suppressions: SuppressionPolicy { suppressions: vec![suppression] },
        ..Command_At(root)
    };
}

/// A [`Suppression`] matching `finding` exactly, with a disposition, rationale and owner
/// fixed for every fixture that reaches for one -- what a test needs is that it addresses a
/// specific real finding, never what the suppression itself says.
fn Suppression_Of(finding: &Finding) -> Suppression
{
    return Suppression {
        rule: finding.rule.clone(),
        subject: finding.subject,
        disposition: SuppressionDisposition::FalsePositiveDisposition,
        rationale: "test fixture".to_owned(),
        owner: "test".to_owned(),
        expiry: None,
    };
}

/// [`Command_At`] `root`, with `debt` as the whole baseline policy and every rule still
/// selected -- the shape both a `run` and an `explain` fixture build once they have a real
/// finding to baseline.
fn Command_With_Baseline(root: PathBuf, debt: BaselineDebt) -> GateCommand
{
    return GateCommand { baseline: BaselinePolicy { debt: vec![debt] }, ..Command_At(root) };
}

/// A [`BaselineDebt`] matching `finding` exactly, with a rationale fixed for every fixture
/// that reaches for one -- the same "what a test needs is that it addresses a specific real
/// finding" discipline [`Suppression_Of`] already keeps.
fn Baseline_Of(finding: &Finding) -> BaselineDebt
{
    return BaselineDebt {
        rule: finding.rule.clone(),
        subject: finding.subject,
        rationale: "test fixture".to_owned(),
        // The state every entry authored before `OD-GATE-030` v2 is in, so these fixtures go
        // on asserting exactly what they asserted before the quantity existed.
        allowance: BaselineAllowance::Unbounded,
        // `None` because this entry is built in code and names no file, which is the state a
        // caller-authored policy is in. A fixture that wanted to assert what a *declared*
        // entry does with its path has to go through `gate_policy_file` to get one.
        declared_path: None,
    };
}

/// [`Command_At`] `root`, with `calibration` as the whole adoption policy and every rule
/// still selected -- the shape both a `run` and an `explain` fixture build once they have a
/// real finding whose rule to calibrate.
fn Command_With_Calibration(root: PathBuf, calibration: RuleCalibration) -> GateCommand
{
    return GateCommand { adoption: AdoptionPolicy { calibrated: vec![calibration] }, ..Command_At(root) };
}

/// A [`RuleCalibration`] matching `finding`'s rule exactly, with a rationale fixed for every
/// fixture that reaches for one -- the same "what a test needs is that it addresses a
/// specific real finding" discipline [`Suppression_Of`] and [`Baseline_Of`] both keep.
fn Calibration_Of(finding: &Finding) -> RuleCalibration
{
    return RuleCalibration { rule: finding.rule.clone(), rationale: "test fixture".to_owned() };
}

/// Judges one fresh call of `source` with every policy empty, and returns its one real
/// blocking finding -- the shared setup a suppression and a baseline fixture both need
/// before either can address a specific finding with its own policy.
fn One_Real_Blocking_Finding(source: impl Fn() -> SourceFile) -> Finding
{
    let unmatched = Ran_Over(vec![source()], &Command_At(Repository_Root()));

    return unmatched
        .findings
        .blocking_findings
        .into_iter()
        .next()
        .expect("this fixture must produce one real blocking finding");
}

/// Asserts `result` reports no finding able to fail the build, and that `tolerated` --
/// whichever of `result.findings.suppressed_findings`/`result.findings.baselined_findings` the caller's own
/// policy populated -- is not empty, so the one real finding a suppression or a baseline
/// fixture produces is visible somewhere rather than silently disappearing.
fn Assert_Tolerated_Not_Blocking(result: &GateRunResult, tolerated: &[Finding])
{
    assert!(matches!(result.check_outcome, nomos_check_orchestration::CheckOutcome::Judged { .. }));
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    assert!(result.findings.blocking_findings.is_empty(), "{:?}", result.findings.blocking_findings);
    assert!(!tolerated.is_empty(), "the tolerated finding must still be visible");
}

/// A source no provider could materialize a fact for makes this run's own recomputed
/// `Claim` `Incomplete` -- the fixture every coverage-policy test builds on.
/// Proven once here rather than assumed at each call site: `Claim::Incomplete` is the
/// premise, not the thing under test, for every fixture that reuses this text.
fn Coverage_Debt_Fixture() -> Vec<SourceFile>
{
    return vec![Clean_Source(SourcePath("a.rs")), Source_File(SourcePath("broken.rs"), SourceText("pub const ??? = ;"))];
}
