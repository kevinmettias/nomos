//! A real `Run_Gate`, run four times over one scratch tree, told apart by its own record.
//!
//! `occurrence_history::tests` proves what a record establishes from hand-built states. This
//! proves the half that cannot be asserted there: that a run actually reads the file, actually
//! writes it back, and that what the record establishes actually moves a finding out of
//! `baselined_findings` when the tolerance stops reaching it.

use super::occurrence_history::GATE_HISTORY_FILE;
use super::Run_Gate;
use crate::{
    BaselineAllowance, BaselineDebt, BaselinePolicy, GateCommand, GateEnvironment, GateRunOutcome, GateRunResult,
};
use nomos_contracts::{Digest128, Finding, RunId};
use nomos_model::Subject_Of_Path;
use nomos_platform::Timestamp;
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProgramLauncher};
use nomos_rules::SourceFile;
use nomos_workspace::BuildVariant;
use std::path::{Path, PathBuf};

/// The file the fixtures below judge, and the one a baseline entry is written over.
const SOURCE_PATH: &str = "a.rs";

/// One source `Check_Completeness_Mirrors` reports a real blocking finding for: a constant whose
/// doc claims a mirror no fixture here declares.
const VIOLATING_SOURCE: &str = "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n";

/// The same file with the violation fixed, which is what makes the occurrence go away.
const FIXED_SOURCE: &str = "pub const TABLE: &[&str] = &[];\n";

/// The record a repository opts in with. Its presence is the whole opt-in, so a fixture that
/// forgot to write it would be testing a run that records nothing.
const AN_EMPTY_RECORD: &str = "{\"scopes\": []}";

/// A scratch root of its own, carrying [`AN_EMPTY_RECORD`] and nothing else.
///
/// Named per test because these runs write into their root, and two tests sharing one would each
/// be advancing the other's states.
fn Opted_In_Root(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-gate-continuity-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("the scratch root sits under std::env::temp_dir(), which every platform this runs on provides");
    std::fs::write(root.join(GATE_HISTORY_FILE), AN_EMPTY_RECORD)
        .expect("the directory create_dir_all just returned Ok for is where this writes");

    return root;
}

/// The build variant every fixture judges as.
fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// A fixed run identity: these tests judge findings and buckets, not identities.
fn Test_Run_Id() -> RunId
{
    return RunId::From_Digest(Digest128::From_Bytes([0; Digest128::BYTE_LENGTH]));
}

/// [`SOURCE_PATH`] carrying `text`, with the subject a real walk would give it.
fn Source(text: &str) -> Vec<SourceFile>
{
    return vec![SourceFile::New(SOURCE_PATH, Subject_Of_Path(SOURCE_PATH), text.to_owned())];
}

/// `sources` judged at `root` under `baseline`, through the real platform.
fn Ran(root: &Path, sources: Vec<SourceFile>, baseline: BaselinePolicy) -> GateRunResult
{
    let command = GateCommand { root: root.to_path_buf(), baseline, ..Default::default() };
    let environment = GateEnvironment {
        variant: Test_Variant(),
        launcher: &StdProgramLauncher,
        filesystem: &StdFileSystem,
        environment: &StdEnvironment,
        now: Timestamp::From_Unix_Seconds(0),
    };

    return Run_Gate(Some(sources), environment, &command, Test_Run_Id());
}

/// A baseline over `finding`'s own scope, unbounded -- the state `OD-GATE-030` says every entry
/// authored before it is in, so the counting bound tolerates whatever the scope holds and the
/// record is the only thing that can withdraw the tolerance.
fn Baseline_Over(finding: &Finding) -> BaselinePolicy
{
    return BaselinePolicy {
        debt: vec![BaselineDebt {
            rule: finding.rule.clone(),
            subject: finding.subject,
            rationale: "test fixture".to_owned(),
            allowance: BaselineAllowance::Unbounded,
            declared_path: None,
        }],
    };
}

/// The one blocking finding the violating source produces, read off a real run rather than
/// constructed, so the baseline entry below addresses what this workspace's rules actually emit.
fn A_Real_Finding(root: &Path) -> Finding
{
    let probe = Ran(root, Source(VIOLATING_SOURCE), BaselinePolicy::default());

    return probe
        .findings
        .blocking_findings
        .first()
        .cloned()
        .expect("the violating fixture claims a mirror nothing declares, which is a blocking finding");
}

/// The whole loop, in the order a repository would live it.
///
/// Four runs and one assertion each, because the sequence *is* the claim: a tolerance that is
/// undetermined at first, becomes continuity once there is an interval to cross, and stops
/// reaching the occurrence once the record has seen it go and come back. Split into four tests
/// each would need three runs of setup apiece, and the setup would be this test.
#[test]
fn Test_A_Tolerance_Should_Survive_An_Edit_And_Not_Survive_A_Reintroduction()
{
    let root = Opted_In_Root("reintroduction");
    let baseline = Baseline_Over(&A_Real_Finding(&root));

    let first = Ran(&root, Source(VIOLATING_SOURCE), baseline.clone());
    assert_eq!(first.findings.baselined_findings.len(), 1, "the entry tolerates it, and one state establishes nothing to withdraw that");
    assert_eq!(first.disposition, GateRunOutcome::Passed);

    let second = Ran(&root, Source(VIOLATING_SOURCE), baseline.clone());
    assert_eq!(second.findings.baselined_findings.len(), 1, "still there across an interval the record now covers");
    assert_eq!(second.disposition, GateRunOutcome::Passed);

    let fixed = Ran(&root, Source(FIXED_SOURCE), baseline.clone());
    assert!(fixed.findings.baselined_findings.is_empty(), "the violation is gone: {:?}", fixed.findings.baselined_findings);

    let reintroduced = Ran(&root, Source(VIOLATING_SOURCE), baseline);
    assert!(
        reintroduced.findings.baselined_findings.is_empty(),
        "a reintroduction must not be reported as baselined debt: {:?}",
        reintroduced.findings.baselined_findings
    );
    assert_eq!(reintroduced.findings.blocking_findings.len(), 1, "it blocks, because a tolerance does not survive a recreation");
    assert_eq!(reintroduced.disposition, GateRunOutcome::Failed);
}

/// A run over a tree with no record changes nothing and leaves no file behind, which is what
/// keeps every caller that predates this unchanged.
#[test]
fn Test_A_Tree_With_No_Record_Should_Not_Acquire_One()
{
    let root = std::env::temp_dir().join(format!("nomos-gate-continuity-unopted-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("the scratch root sits under std::env::temp_dir(), which every platform this runs on provides");
    let baseline = Baseline_Over(&A_Real_Finding(&root));

    let result = Ran(&root, Source(VIOLATING_SOURCE), baseline);

    assert!(!root.join(GATE_HISTORY_FILE).exists(), "the presence of the record is the opt-in, and a run must not write one nobody asked for");
    assert_eq!(result.findings.baselined_findings.len(), 1, "and the tolerance behaves exactly as it did before any of this existed");
}

/// The record a run leaves behind is readable and says what it established, which is the report
/// this whole mechanism is for.
#[test]
fn Test_A_Run_Should_Leave_A_Record_Naming_What_It_Established()
{
    let root = Opted_In_Root("report");
    let baseline = Baseline_Over(&A_Real_Finding(&root));

    Ran(&root, Source(VIOLATING_SOURCE), baseline.clone());
    Ran(&root, Source(VIOLATING_SOURCE), baseline);

    let written = std::fs::read_to_string(root.join(GATE_HISTORY_FILE)).expect("the run above wrote the record it read");

    assert!(written.contains("\"continuity\": \"ExactContinuity\""), "{written}");
    assert!(written.contains("completeness-mirror"), "a reader must find the rule that made the claim: {written}");
}

