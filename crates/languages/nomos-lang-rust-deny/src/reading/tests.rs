//! What this module promises, exercised — the launchers, scratch directories and JSON
//! fixtures `Discover_Workspace` and its helpers are read through.

use nomos_platform_std::StdEnvironment;
use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use super::*;
use nomos_platform::ProcessOutput;
use std::cell::RefCell;

/// A launcher that hands `Discover_Workspace` a fixed stderr stream instead of running
/// a real `cargo deny` — the boundary this crate's own module doc names as the one
/// place a caller substitutes a real subprocess.
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

/// A launcher that records every [`Command`] it was asked to run, rather than
/// actually running one — what [`Test_Discover_Workspace_Should_Pin_Cargo_Denys_Own_
/// Config_Discovery_To_Roots_Own_Deny_Toml`] asserts the exact argv of, and what
/// [`Test_Discover_Workspace_Should_Refuse_Before_Launching_When_Root_Has_No_Deny_
/// Toml`] asserts is never invoked at all. `ProcessLauncher::Run` takes `&self`, so
/// recording needs interior mutability rather than a `&mut self` this trait does not
/// offer.
struct RecordingLauncher
{
    received: RefCell<Vec<Command>>,
}

impl RecordingLauncher
{
    fn New() -> Self
    {
        return Self { received: RefCell::new(Vec::new()) };
    }
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for RecordingLauncher
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProcessLauncher for RecordingLauncher
{
    fn Run(&self, command: &Command) -> Result<ProcessOutput, String>
    {
        self.received.borrow_mut().push(command.clone());

        return Ok(ProcessOutput {
            outcome: ExitOutcome::Exited { code: 0 },
            stdout: String::new(),
            stderr: Completed_Summary(),
        });
    }
}

/// The summary line a completed `cargo deny check bans licenses sources` really writes,
/// in the shape this workspace's own run produces it.
///
/// Every fixture that means "a run that finished" carries this, because
/// [`Require_Summarized`] is what separates a finished run from an unreadable one and a
/// fixture without it is asserting about the refusal path rather than the parse.
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

/// A real scratch directory, removed when the test ends -- [`Required_Deny_Config`]
/// reads the real filesystem (`root.join("deny.toml").is_file()`), so proving it needs a
/// real directory rather than a hardcoded, possibly-nonexistent path.
struct ScratchDirectory
{
    root: PathBuf,
}

impl ScratchDirectory
{
    fn New(name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-lang-rust-deny-config-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");

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

/// `P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT`: a root with no `deny.toml` of its
/// own must be refused before `cargo deny` is ever launched, not silently judged
/// against whichever ancestor's `deny.toml` `cargo deny`'s own upward search would
/// otherwise find.
#[test]
fn Test_Discover_Workspace_Should_Refuse_Before_Launching_When_Root_Has_No_Deny_Toml()
{
    let scratch = ScratchDirectory::New("missing");
    let launcher = RecordingLauncher::New();

    let error = Discover_Workspace(scratch.Path(), &launcher, &StdEnvironment)
        .expect_err("a root with no deny.toml of its own must be refused");

    assert!(error.reason.contains("deny.toml"), "{}", error.reason);
    assert!(
        launcher.received.borrow().is_empty(),
        "cargo deny must never be launched when this reader could not confirm a deny.toml \
         of root's own exists: {:?}",
        launcher.received.borrow()
    );
}

/// A root that does have its own `deny.toml` must have `cargo deny` pinned to exactly
/// that file via `--config`, not left to its own upward search -- the fix half of the
/// same escape [`Test_Discover_Workspace_Should_Refuse_Before_Launching_When_Root_Has_
/// No_Deny_Toml`] proves the refusal half of.
#[test]
fn Test_Discover_Workspace_Should_Pin_Cargo_Denys_Own_Config_Discovery_To_Roots_Own_Deny_Toml()
{
    let scratch = ScratchDirectory::New("present");
    std::fs::write(scratch.Path().join("deny.toml"), "[bans]\nmultiple-versions = \"allow\"\n").expect("writing a scratch deny.toml");
    let launcher = RecordingLauncher::New();

    let _ignored = Discover_Workspace(scratch.Path(), &launcher, &StdEnvironment).expect("a root with its own deny.toml must be run, not refused");

    let received = launcher.received.borrow();
    let command = received.first().expect("Discover_Workspace must have launched exactly one command");
    let expected_config = scratch.Path().join("deny.toml").to_string_lossy().into_owned();
    let config_index = command
        .argv
        .iter()
        .position(|argument| return argument == "--config")
        .expect("the launched command must carry --config");
    assert_eq!(
        command.argv.get(config_index.saturating_add(1)),
        Some(&expected_config),
        "--config must name root's own deny.toml exactly: {:?}",
        command.argv
    );
}

/// The root is a real scratch directory carrying its own `deny.toml`, not a hand-typed
/// literal, because [`Discover_Workspace`] refuses before launching anything unless
/// `root`'s own `deny.toml` exists on disk.
///
/// It used to root at `F:/repos/nomos`, where that file is real on one developer's
/// machine and nowhere else, so the test passed there and failed on ubuntu with the
/// refusal rather than the parse it means to exercise. `P80`.
#[test]
fn Test_Discover_Workspace_Should_Read_A_Violation_From_The_Json_Stream()
{
    let scratch = ScratchDirectory::New("violation-stream");
    std::fs::write(scratch.Path().join("deny.toml"), "[bans]\nmultiple-versions = \"warn\"\n").expect("writing a scratch deny.toml");
    let stderr = serde_json::json!({
        "type": "diagnostic",
        "fields": {
            "code": "duplicate",
            "severity": "warning",
            "message": "multiple versions",
            "labels": [],
            "graphs": []
        }
    })
    .to_string();
    let stderr = format!("{stderr}\n{}", Completed_Summary());
    let launcher = FakeLauncher { stderr };

    let violations = Discover_Workspace(scratch.Path(), &launcher, &StdEnvironment).expect("the fake launcher writes a real stderr stream");

    assert_eq!(violations.len(), 1);
    assert_eq!(violations.first().expect("one violation").code, "duplicate");
}

/// Rooted at a real scratch directory with its own `deny.toml` for the same reason as
/// the test above, and for a sharper one here: without that file the refusal this
/// asserts arrives, but it is the *wrong refusal*. On ubuntu it failed claiming no
/// `deny.toml`, never reaching the empty-stream case the test is named for, so a
/// passing run on Windows was evidence about a different code path. `P80`.
#[test]
fn Test_Discover_Workspace_Should_Refuse_An_Empty_Stderr_Stream()
{
    let scratch = ScratchDirectory::New("empty-stream");
    std::fs::write(scratch.Path().join("deny.toml"), "[bans]\nmultiple-versions = \"warn\"\n").expect("writing a scratch deny.toml");
    let launcher = FakeLauncher { stderr: String::new() };

    let error = Discover_Workspace(scratch.Path(), &launcher, &StdEnvironment).expect_err("an empty stderr stream is not a real cargo deny run");

    assert!(error.reason.contains("no output"), "{}", error.reason);
}

/// The case that was silently clean before `P81`, and the one no test covered: stderr
/// that is not empty, so the emptiness guard passes, and carries nothing readable, so
/// every line is skipped and no violation survives.
///
/// This is what a `cargo deny` that dies before finishing looks like from here. It used
/// to return `Ok` of an empty vector, indistinguishable from a workspace with no policy
/// violations at all, and `Materialize_Workspace` published it as a clean fact.
#[test]
fn Test_Discover_Workspace_Should_Refuse_A_Non_Empty_Stream_Carrying_No_Summary()
{
    let scratch = ScratchDirectory::New("unreadable-stream");
    std::fs::write(scratch.Path().join("deny.toml"), "[bans]\nmultiple-versions = \"warn\"\n").expect("writing a scratch deny.toml");
    let launcher = FakeLauncher {
        stderr: "error: failed to fetch the advisory database\nnote: run with --offline\n".to_owned(),
    };

    let error = Discover_Workspace(scratch.Path(), &launcher, &StdEnvironment).expect_err(
        "a stream with no summary means cargo deny never finished its checks, and an empty violation list over it is not a clean result",
    );

    assert!(error.reason.contains("no summary"), "{}", error.reason);
}

/// A run that finished *some* checks is not a run that finished the ones this provider
/// asked for, and the silence of a check that never ran is not evidence about it.
#[test]
fn Test_Discover_Workspace_Should_Refuse_A_Summary_That_Skips_A_Requested_Check()
{
    let scratch = ScratchDirectory::New("partial-summary");
    std::fs::write(scratch.Path().join("deny.toml"), "[bans]\nmultiple-versions = \"warn\"\n").expect("writing a scratch deny.toml");
    let launcher = FakeLauncher {
        stderr: serde_json::json!({
            "type": "summary",
            "fields": { "bans": { "errors": 0, "helps": 0, "notes": 0, "warnings": 0 } }
        })
        .to_string(),
    };

    let error = Discover_Workspace(scratch.Path(), &launcher, &StdEnvironment).expect_err("a summary accounting for only one of three requested checks is not a completed run");

    assert!(error.reason.contains("licenses"), "{}", error.reason);
    assert!(error.reason.contains("sources"), "{}", error.reason);
}

/// [`Requested_Checks`] reads the checks back off the command rather than repeating
/// them, so this is the assertion that the two stay the same set.
#[test]
fn Test_Requested_Checks_Should_Name_Every_Check_The_Command_Asks_For()
{
    let command = Cargo_Deny_Command(Path::new("root"), Path::new("root/deny.toml"), &StdEnvironment);

    assert_eq!(Requested_Checks(&command), vec!["bans".to_owned(), "licenses".to_owned(), "sources".to_owned()]);
}

#[test]
fn Test_A_Real_Captured_Diagnostic_Should_Parse()
{
    let diagnostic = serde_json::json!({
        "type": "diagnostic",
        "fields": {
            "code": "license-not-encountered",
            "severity": "warning",
            "message": "license was not encountered",
            "labels": [],
            "graphs": []
        }
    });

    let violation = Violation_Of(&diagnostic).expect("a real diagnostic with all three fields must parse");

    assert_eq!(violation.severity, PolicySeverity::Warning);
    assert_eq!(violation.code, "license-not-encountered");
    assert_eq!(violation.message, "license was not encountered");
}

#[test]
fn Test_A_Summary_Line_Should_Not_Parse_As_A_Violation()
{
    let summary = serde_json::json!({ "type": "summary", "fields": { "errors": 0 } });

    assert_eq!(Violation_Of(&summary), None);
}

#[test]
fn Test_A_Diagnostic_With_No_Code_Should_Not_Parse()
{
    let diagnostic = serde_json::json!({
        "type": "diagnostic",
        "fields": { "severity": "warning", "message": "oops" }
    });

    assert_eq!(Violation_Of(&diagnostic), None);
}

#[test]
fn Test_Violations_Of_Should_Skip_Non_Diagnostic_Lines_And_Malformed_Ones()
{
    let stdout = "not json at all\n\
                   {\"type\": \"summary\", \"fields\": {}}\n\
                   {\"type\": \"diagnostic\", \"fields\": {\"code\": \"duplicate\", \"severity\": \"warning\", \"message\": \"m\"}}\n";

    let violations = Violations_Of(stdout);

    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations.first().expect("asserted len 1 above").code, "duplicate");
}

#[test]
fn Test_Canonical_Order_Should_Sort_By_Severity_Then_Code_Then_Message()
{
    let violations = vec![
        PolicyViolation { severity: PolicySeverity::Warning, code: "zzz".to_owned(), message: "m".to_owned(), target: None },
        PolicyViolation { severity: PolicySeverity::Error, code: "aaa".to_owned(), message: "m".to_owned(), target: None },
        PolicyViolation { severity: PolicySeverity::Warning, code: "aaa".to_owned(), message: "m".to_owned(), target: None },
    ];

    let ordered = Canonical_Order(violations);
    let [first, second, third] = ordered.as_slice()
    else
    {
        // The vector above is a three-element literal a line up; a length mismatch
        // here would mean `Canonical_Order` dropped or duplicated an entry, which is
        // exactly the defect this test exists to catch, not a condition to recover
        // from and report past.
        panic!("asserted three violations above");
    };

    assert_eq!(first.severity, PolicySeverity::Error);
    assert_eq!(second.code, "aaa");
    assert_eq!(third.code, "zzz");
}
