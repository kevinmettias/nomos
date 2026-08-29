//! Running `cargo deny --format json check bans licenses sources` and reading the
//! workspace's own resolved-graph verdict out of the newline-delimited JSON stream it
//! prints, one JSON object per line — verified directly against this workspace's own
//! `cargo deny` output before this reader was written, not assumed from the format's name
//! alone.

use nomos_cap_dependency_policy::{PolicySeverity, PolicyViolation};
use nomos_platform::{Command, ExitOutcome, ProcessLauncher};
use std::path::Path;
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
pub fn Discover_Workspace<P: ProcessLauncher>(root: &Path, launcher: &P) -> Result<Vec<PolicyViolation>, DenyError>
{
    let stream = Run_Cargo_Deny(root, launcher)?;

    return Ok(Canonical_Order(Violations_Of(&stream)));
}

fn Run_Cargo_Deny<P: ProcessLauncher>(root: &Path, launcher: &P) -> Result<String, DenyError>
{
    let command = Cargo_Deny_Command(root);
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

    return Ok(output.stderr);
}

fn Cargo_Deny_Command(root: &Path) -> Command
{
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let mut command = Command::New(
        vec![
            cargo,
            "deny".to_owned(),
            "--format".to_owned(),
            "json".to_owned(),
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
    match outcome
    {
        ExitOutcome::Exited { .. } => return Ok(()),
        ExitOutcome::TimedOut =>
        {
            return Err(DenyError {
                reason: format!("cargo deny was still running after {TIMEOUT:?} and was killed"),
            });
        }
        ExitOutcome::Stalled { idle_elapsed } =>
        {
            return Err(DenyError {
                reason: format!("cargo deny produced no output for {idle_elapsed:?} and was judged stalled"),
            });
        }
        ExitOutcome::Terminated =>
        {
            return Err(DenyError {
                reason: "cargo deny was terminated before it could finish".to_owned(),
            });
        }
    }
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
    use super::*;

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
            panic!("asserted three violations above");
        };

        assert_eq!(first.severity, PolicySeverity::Error);
        assert_eq!(second.code, "aaa");
        assert_eq!(third.code, "zzz");
    }
}
