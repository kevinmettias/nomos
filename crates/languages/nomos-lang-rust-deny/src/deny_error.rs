//! Running `cargo deny --format json check bans licenses sources` and reading the
//! workspace's own resolved-graph verdict out of the newline-delimited JSON stream it
//! prints, one JSON object per line — verified directly against this workspace's own
//! `cargo deny` output before this reader was written, not assumed from the format's name
//! alone.

use nomos_cap_dependency_policy::{PolicySeverity, PolicyViolation};
use nomos_platform::{Command, ExitOutcome, ProcessLauncher};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// `cargo deny check` over this whole repository reads `Cargo.lock` and every crate's own
/// license metadata; warm, it returns in a few seconds. This bound is headroom, not the
/// expected case.
const TIMEOUT: Duration = Duration::from_secs(120);

/// `cargo deny` could not be run or its answer could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenyError
{
    pub reason: String,
}

impl core::fmt::Display for DenyError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// Every violation `cargo deny check bans licenses sources` reported over the whole
/// workspace's resolved dependency graph — empty when it found none, a real and
/// distinguishable "clean" answer.
///
/// # Why this reader reads `stderr`, not `stdout`
///
/// `cargo deny`'s own `--format json` stream is written to **stderr**, not stdout — the
/// same distinction its own `--audit-compatible-output` flag documents by naming what it
/// changes ("output the exact same output as cargo-audit would, to stdout instead of
/// stderr"), and verified directly against this workspace's own real invocation before
/// this reader was written, the same discipline `nomos_lang_rust_clippy::reading`'s own
/// module doc holds itself to for `cargo clippy`'s stream.
///
/// # Why this reader does not gate on `cargo deny`'s own exit code
///
/// `cargo deny check` exits non-zero when any diagnostic reaches `deny.toml`'s own `deny`
/// severity — the same run that produced the real violations this reader exists to
/// relay, not a run that failed to answer. Gating on exit code the way
/// `nomos_lang_rust_clippy::reading::Require_Clean_Exit` does for `cargo clippy` (whose
/// gate step carries no `-D warnings`, so *any* non-zero exit there is a genuine tool
/// failure) would misread `cargo deny`'s own honest "I found what you told me to deny" as
/// this provider being unavailable. What this reader gates on instead is whether `cargo
/// deny` produced its own JSON stream at all: a real run, whatever it found, always
/// writes to stderr; a setup failure (an unreadable `deny.toml`, a workspace `cargo deny`
/// cannot resolve) writes a human-readable error there too, but with no line this reader
/// can parse as a diagnostic.
///
/// # Errors
///
/// [`DenyError`] if the `cargo` binary cannot be run, is killed for exceeding [`TIMEOUT`]
/// or going idle for that long, is terminated before finishing, or produces no stderr at
/// all — the one signal this reader treats as a genuine failure to answer rather than an
/// answer it does not like.
pub fn Discover_Workspace<Launcher: ProcessLauncher>(root: &Path, launcher: &Launcher) -> Result<Vec<PolicyViolation>, DenyError>
{
    let config = Required_Deny_Config(root)?;
    let stream = Run_Cargo_Deny(root, &config, launcher)?;

    return Ok(Canonical_Order(Violations_Of(&stream)));
}

/// `root`'s own `deny.toml`, required to exist before `cargo deny` is ever launched.
///
/// `Cargo_Deny_Command` never used to pass `--config`, so `cargo deny` walked upward from
/// `working_directory` looking for the nearest `deny.toml` on its own -- independently of
/// cargo's own workspace-root resolution, and a second, separate escape from the one
/// `nomos_lang_rust_cargo::metadata_error::Require_Workspace_Root_Is` catches
/// (`P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT`). `tests/integration/fixtures/
/// third-party/hex-0.4.3/deny.toml`'s own doc comment measures this directly: without a
/// `deny.toml` of its own, that fixture's `cargo deny` invocation walked up past it and
/// judged the whole enclosing repository's dependency-license diversity instead. Pinning
/// `--config` to a `deny.toml` this reader confirmed exists at `root` closes that gap from
/// this side too, rather than leaving every caller to supply its own workaround fixture.
fn Required_Deny_Config(root: &Path) -> Result<PathBuf, DenyError>
{
    let config = root.join("deny.toml");
    if !config.is_file()
    {
        return Err(DenyError {
            reason: format!(
                "no deny.toml at {} -- refusing rather than letting cargo deny's own upward \
                 search silently pick up an ancestor's config",
                config.display()
            ),
        });
    }

    return Ok(config);
}

fn Run_Cargo_Deny<Launcher: ProcessLauncher>(root: &Path, config: &Path, launcher: &Launcher) -> Result<String, DenyError>
{
    let command = Cargo_Deny_Command(root, config);
    let output = launcher.Run(&command).map_err(|error| DenyError {
        reason: format!("cargo deny could not be run: {error}"),
    })?;

    Require_Ran(&output.outcome)?;

    if output.stderr.trim().is_empty()
    {
        return Err(DenyError {
            reason: format!("cargo deny produced no output to read on stderr; stdout: {}", output.stdout),
        });
    }

    Require_Summarized(&output.stderr, &command)?;

    return Ok(output.stderr);
}

/// Every check `command`'s own argv asks `cargo deny` to run.
///
/// Read back off the command rather than declared a second time, so the set of checks this
/// provider requests has exactly one home in [`Cargo_Deny_Command`]'s own argv. A second
/// list here would be two artifacts independently determining one fact, which is the shape
/// `OD-RULES-025` exists over.
fn Requested_Checks(command: &Command) -> Vec<String>
{
    let Some(position) = command.argv.iter().position(|argument| return argument == "check")
    else
    {
        return Vec::new();
    };

    return command.argv.iter().skip(position.saturating_add(1)).cloned().collect();
}

/// What `cargo deny` itself said went wrong, when it wrote a log line saying so.
///
/// Beside its diagnostics and its summary, `cargo deny --format json` writes
/// `{"type":"log"}` lines carrying a `level` and a `message`. A run that dies early says
/// why in one of those and nowhere else -- measured against a directory holding a
/// `deny.toml` and no `Cargo.toml`: one `ERROR` log reading "the directory ... doesn't
/// contain a Cargo.toml file", exit 1, and no summary. Quoting it makes the refusal name
/// the real cause instead of handing a reader the whole stream to search.
fn Logged_Error(stream: &str) -> Option<String>
{
    return stream
        .lines()
        .filter_map(|line| return serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|value| return value.get("type").and_then(serde_json::Value::as_str) == Some("log"))
        .filter(|value| return value.pointer("/fields/level").and_then(serde_json::Value::as_str) == Some("ERROR"))
        .find_map(|value| return value.pointer("/fields/message").and_then(serde_json::Value::as_str).map(|message| return format!("cargo deny reported: {message}")));
}

/// Refuses a stream carrying no completed-run summary that accounts for every check
/// `command` asked for.
///
/// `cargo deny --format json` ends a completed run with one `{"type":"summary"}` line whose
/// `fields` names each check it finished and the counts it reached -- measured directly
/// against this workspace: `{"fields":{"bans":{...},"licenses":{...},"sources":{...}},
/// "type":"summary"}`. A run that dies before finishing writes prose or an error and no
/// such line.
///
/// Without this, that difference was invisible. [`Require_Ran`] passes any completed
/// process whatever its exit code, deliberately and for a real reason of its own; the
/// emptiness guard above catches only a *silent* failure; and [`Violations_Of`] skips every
/// line it cannot read, equally deliberately, because real runs interleave noise. Each is
/// defensible alone, and together they made "checked everything, found nothing" and
/// "failed, and nothing readable came back" the same value -- `Ok(vec![])`, which
/// `Materialize_Workspace` publishes as a clean dependency-policy fact. A supply-chain
/// provider reporting clean because it could not read its own tool is exactly the
/// false-coverage shape `OD-COMPLETENESS-001` exists to refuse. `P81`.
///
/// The summary is what is judged rather than the exit code, because the exit code genuinely
/// cannot carry this: `cargo deny` exits non-zero whenever a diagnostic reaches `deny` --
/// this workspace's own real run exits 4 while succeeding completely.
fn Require_Summarized(stream: &str, command: &Command) -> Result<(), DenyError>
{
    let summary = stream
        .lines()
        .filter_map(|line| return serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|value| return value.get("type").and_then(serde_json::Value::as_str) == Some("summary"));

    let Some(summary) = summary
    else
    {
        let said = Logged_Error(stream).unwrap_or_else(|| return format!("stderr: {stream}"));

        return Err(DenyError {
            reason: format!("cargo deny wrote no summary line, so it did not finish its checks and an empty violation list over it is not a clean result -- {said}"),
        });
    };

    let unaccounted: Vec<String> = Requested_Checks(command)
        .into_iter()
        .filter(|check| return summary.get("fields").and_then(|fields| return fields.get(check)).is_none())
        .collect();

    if !unaccounted.is_empty()
    {
        return Err(DenyError {
            reason: format!("cargo deny's own summary accounts for none of {unaccounted:?}, so those checks did not finish and their silence is not evidence of a clean result; stderr: {stream}"),
        });
    }

    return Ok(());
}

/// `config` is passed explicitly rather than left for `cargo deny` to discover on its own
/// -- [`Required_Deny_Config`]'s own doc explains why that discovery is a second escape
/// this provider must not leave open.
fn Cargo_Deny_Command(root: &Path, config: &Path) -> Command
{
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let mut command = Command::New(
        vec![
            cargo,
            "deny".to_owned(),
            "--format".to_owned(),
            "json".to_owned(),
            "--config".to_owned(),
            config.to_string_lossy().into_owned(),
            "check".to_owned(),
            "bans".to_owned(),
            "licenses".to_owned(),
            "sources".to_owned(),
        ],
        TIMEOUT,
    );
    command.working_directory = Some(root.to_path_buf());

    return command;
}

/// Refuses every outcome that means nobody found out whether `cargo deny` could answer at
/// all -- a timeout, a stall, or a termination. A completed run's own exit code is not
/// judged here; [`Discover_Workspace`]'s own doc explains why.
fn Require_Ran(outcome: &ExitOutcome) -> Result<(), DenyError>
{
    return match outcome
    {
        ExitOutcome::Exited { .. } => Ok(()),
        ExitOutcome::TimedOut => Err(DenyError {
            reason: format!("cargo deny was still running after {TIMEOUT:?} and was killed"),
        }),
        ExitOutcome::Stalled { idle_elapsed } => Err(DenyError {
            reason: format!("cargo deny produced no output for {idle_elapsed:?} and was judged stalled"),
        }),
        ExitOutcome::Terminated => Err(DenyError {
            reason: "cargo deny was terminated before it could finish".to_owned(),
        }),
    };
}

/// `violations`, in a stable order -- neither `cargo deny`'s own internal iteration over
/// the resolved graph nor `serde_json`'s object representation promises one, the same
/// reason `nomos_lang_rust_clippy::reading::Canonical_Order` sorts before encoding.
fn Canonical_Order(mut violations: Vec<PolicyViolation>) -> Vec<PolicyViolation>
{
    violations.sort_by(|left, right| {
        return (left.severity.Label(), &left.code, &left.message).cmp(&(right.severity.Label(), &right.code, &right.message));
    });

    return violations;
}

/// `stderr`'s own JSON-lines stream, folded into every `"type": "diagnostic"` entry it
/// named -- silently skipping a malformed line, a `"type": "summary"` line, and anything
/// missing the three fields every real diagnostic carries, the same tolerance
/// `nomos_lang_rust_clippy::reading::Grouped_By_Package` has for a stream whose other
/// lines are not this reader's concern.
fn Violations_Of(stream: &str) -> Vec<PolicyViolation>
{
    let mut violations = Vec::new();

    for line in stream.lines()
    {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line)
        else
        {
            continue;
        };

        if value.get("type").and_then(serde_json::Value::as_str) != Some("diagnostic")
        {
            continue;
        }

        if let Some(violation) = Violation_Of(&value)
        {
            violations.push(violation);
        }
    }

    return violations;
}

/// One violation out of `cargo deny`'s own `{"type": "diagnostic", "fields": {...}}`
/// object. `None` for a diagnostic missing a severity, a code, or a message -- the three
/// fields every real diagnostic this reader has observed carries unconditionally; a
/// violation missing one of them is not a fact this capability's own contract promises.
fn Violation_Of(diagnostic: &serde_json::Value) -> Option<PolicyViolation>
{
    let fields = diagnostic.get("fields")?;
    let severity = PolicySeverity::From_Label(fields.get("severity")?.as_str()?)?;
    let code = fields.get("code")?.as_str()?.to_owned();
    let message = fields.get("message")?.as_str()?.to_owned();

    return Some(PolicyViolation { severity, code, message });
}

#[cfg(test)]
mod tests
{
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

    /// A real scratch directory, removed when the test ends -- [`Required_Deny_Config`]
    /// now reads the real filesystem (`root.join("deny.toml").is_file()`), so proving it
    /// needs a real directory rather than a hardcoded, possibly-nonexistent path.
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

        let error = Discover_Workspace(scratch.Path(), &launcher)
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

        let _ignored = Discover_Workspace(scratch.Path(), &launcher).expect("a root with its own deny.toml must be run, not refused");

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

        let violations = Discover_Workspace(scratch.Path(), &launcher).expect("the fake launcher writes a real stderr stream");

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

        let error = Discover_Workspace(scratch.Path(), &launcher).expect_err("an empty stderr stream is not a real cargo deny run");

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

        let error = Discover_Workspace(scratch.Path(), &launcher).expect_err(
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

        let error = Discover_Workspace(scratch.Path(), &launcher).expect_err("a summary accounting for only one of three requested checks is not a completed run");

        assert!(error.reason.contains("licenses"), "{}", error.reason);
        assert!(error.reason.contains("sources"), "{}", error.reason);
    }

    /// [`Requested_Checks`] reads the checks back off the command rather than repeating
    /// them, so this is the assertion that the two stay the same set.
    #[test]
    fn Test_Requested_Checks_Should_Name_Every_Check_The_Command_Asks_For()
    {
        let command = Cargo_Deny_Command(Path::new("root"), Path::new("root/deny.toml"));

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
            PolicyViolation { severity: PolicySeverity::Warning, code: "zzz".to_owned(), message: "m".to_owned() },
            PolicyViolation { severity: PolicySeverity::Error, code: "aaa".to_owned(), message: "m".to_owned() },
            PolicyViolation { severity: PolicySeverity::Warning, code: "aaa".to_owned(), message: "m".to_owned() },
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
}
