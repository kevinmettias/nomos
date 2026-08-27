//! Running `cargo clippy --message-format=json` and reading a workspace's own first-party
//! lint diagnostics out of the newline-delimited JSON stream it prints, one JSON object per
//! line — a real, structural difference from `cargo metadata --format-version 1`'s one
//! document, verified directly against this workspace's own `cargo clippy` output before
//! this reader was written, not assumed from the format's name alone.

use nomos_cap_lint::{DiagnosticsPayload, LintDiagnostic, LintLevel};
use nomos_platform::{Command, ExitOutcome, ProcessLauncher};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

mod discovered_diagnostics;

pub use discovered_diagnostics::DiscoveredDiagnostics;

/// `cargo clippy --workspace --all-targets` over this whole repository, cold, can take
/// minutes rather than seconds — every crate is checked under clippy's own lint pass, not
/// merely read like `cargo metadata`'s manifest scan. Warm (the ordinary case, since this
/// provider's own facts are keyed by generation and re-asked for on every run), it returns
/// in a few seconds; this bound is headroom for the cold case, not the expected one.
const TIMEOUT: Duration = Duration::from_secs(600);

/// `cargo clippy` could not be run or its answer could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClippyError
{
    pub reason: String,
}

impl core::fmt::Display for ClippyError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// Every first-party workspace member's own lint diagnostics, one entry per member —
/// present with an empty list for a member `cargo clippy` found nothing to report about,
/// the same "clean is a real answer" shape [`DiagnosticsPayload`] itself already commits
/// to. A member is discovered from `cargo clippy`'s own JSON stream directly: every
/// first-party package this run touches reports at least a `compiler-artifact` entry
/// whether or not it also reports a `compiler-message`, so no separate `cargo metadata`
/// call is needed to enumerate the set — deliberately, since `nomos-lang-rust-clippy` and
/// `nomos-lang-rust-cargo` share a band and neither may name the other.
///
/// # Errors
///
/// [`ClippyError`] if the `cargo` binary cannot be run, exits non-zero, is killed for
/// exceeding [`TIMEOUT`] or going idle for that long, or its stdout could not be read as
/// the newline-delimited JSON stream `--message-format json` promises.
pub fn Discover_Workspace<P: ProcessLauncher>(root: &Path, launcher: &P) -> Result<Vec<DiscoveredDiagnostics>, ClippyError>
{
    let stdout = Run_Cargo_Clippy(root, launcher)?;
    let discovered = Grouped_By_Package(&stdout, root);

    return Require_Nonempty(discovered);
}

fn Run_Cargo_Clippy<P: ProcessLauncher>(root: &Path, launcher: &P) -> Result<String, ClippyError>
{
    let command = Cargo_Clippy_Command(root);
    let output = launcher.Run(&command).map_err(|error| ClippyError {
        reason: format!("cargo clippy could not be run: {error}"),
    })?;

    Require_Clean_Exit(&output.outcome, &output.stderr)?;

    return Ok(output.stdout);
}

/// The same invocation `.github/workflows/gate.yml`'s own lint step runs
/// (`--workspace --all-targets`), so this provider's own diagnostics are never a different
/// question from the one CI already gates on -- plus `--message-format json` for a
/// machine-readable stream in place of the gate's own human-rendered text.
fn Cargo_Clippy_Command(root: &Path) -> Command
{
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let mut command = Command::New(
        vec![
            cargo,
            "clippy".to_owned(),
            "--workspace".to_owned(),
            "--all-targets".to_owned(),
            "--message-format".to_owned(),
            "json".to_owned(),
        ],
        TIMEOUT,
    );
    command.working_directory = Some(root.to_path_buf());

    return command;
}

/// Refuses every outcome a launched process can report other than a clean, zero exit —
/// the same shape `nomos_lang_rust_cargo`'s own `Require_Clean_Exit` uses for the identical
/// reason: a real compile error, not merely a lint warning, is the only thing that makes
/// `cargo clippy` exit non-zero here, since this workspace's own gate step carries no
/// `-D warnings`.
fn Require_Clean_Exit(outcome: &ExitOutcome, stderr: &str) -> Result<(), ClippyError>
{
    match outcome
    {
        ExitOutcome::Exited { code: 0 } => return Ok(()),
        ExitOutcome::Exited { code } =>
        {
            return Err(ClippyError {
                reason: format!("cargo clippy failed (exit {code}): {stderr}"),
            });
        }
        ExitOutcome::TimedOut =>
        {
            return Err(ClippyError {
                reason: format!("cargo clippy was still running after {TIMEOUT:?} and was killed"),
            });
        }
        ExitOutcome::Stalled { idle_elapsed } =>
        {
            return Err(ClippyError {
                reason: format!("cargo clippy produced no output for {idle_elapsed:?} and was judged stalled"),
            });
        }
        ExitOutcome::Terminated =>
        {
            return Err(ClippyError {
                reason: "cargo clippy was terminated before it could finish".to_owned(),
            });
        }
    }
}

/// `stdout`'s own JSON-lines stream, folded into one entry per first-party workspace
/// member the stream named at all -- by a `compiler-artifact`, a `compiler-message`, or
/// both.
fn Grouped_By_Package(stdout: &str, root: &Path) -> Vec<DiscoveredDiagnostics>
{
    let mut members: BTreeMap<String, (String, Vec<LintDiagnostic>)> = BTreeMap::new();

    for line in stdout.lines()
    {
        Record_Line(line, root, &mut members);
    }

    return members
        .into_iter()
        .map(|(manifest_relative_root, (package, diagnostics))| {
            return DiscoveredDiagnostics {
                payload: DiagnosticsPayload { package, diagnostics: Canonical_Order(diagnostics) },
                manifest_relative_root,
            };
        })
        .collect();
}

/// One line of `cargo clippy`'s own JSON-lines stream, folded into `members` if it names a
/// first-party workspace package.
fn Record_Line(line: &str, root: &Path, members: &mut BTreeMap<String, (String, Vec<LintDiagnostic>)>)
{
    let Ok(value) = serde_json::from_str::<serde_json::Value>(line)
    else
    {
        return;
    };

    let Some(relative_root) = First_Party_Relative_Root_Of(&value, root)
    else
    {
        return;
    };

    let name = Package_Name(&relative_root);
    let entry = members.entry(relative_root).or_insert_with(|| return (name, Vec::new()));

    Record_Compiler_Diagnostic(&value, entry);
}

/// `value`'s `package_id`, resolved to a first-party workspace member's root -- [`Record_Line`]'s
/// own first two guard clauses, named so its body reads as one decision per line.
fn First_Party_Relative_Root_Of(value: &serde_json::Value, root: &Path) -> Option<String>
{
    let package_id = value.get("package_id").and_then(serde_json::Value::as_str)?;

    return First_Party_Relative_Root(package_id, root);
}

/// `value`'s compiler-message diagnostic, appended to `entry` if it is one -- [`Record_Line`]'s
/// own trailing step, named so a message that is not a compiler diagnostic reads as "nothing
/// to append" rather than as a condition guarding the whole function.
fn Record_Compiler_Diagnostic(value: &serde_json::Value, entry: &mut (String, Vec<LintDiagnostic>))
{
    if value.get("reason").and_then(serde_json::Value::as_str) == Some("compiler-message")
        && let Some(message) = value.get("message")
        && let Some(diagnostic) = Diagnostic_Of(message)
    {
        entry.1.push(diagnostic);
    }
}

/// `diagnostics`, deduplicated and in a stable order — `--all-targets` compiles a member's
/// own library crate more than once (its plain build, its own test binary), and a real
/// warning at one line is reported once per compilation that reaches it, which is the same
/// diagnostic reported twice rather than two diagnostics. Sorting first is what makes
/// `Vec::dedup`'s consecutive-equality rule catch every duplicate rather than only
/// adjacent ones the stream happened to emit next to each other.
fn Canonical_Order(mut diagnostics: Vec<LintDiagnostic>) -> Vec<LintDiagnostic>
{
    diagnostics.sort_by(|left, right| {
        return (&left.file, left.line, left.level.Label(), &left.lint, &left.message)
            .cmp(&(&right.file, right.line, right.level.Label(), &right.lint, &right.message));
    });
    diagnostics.dedup();

    return diagnostics;
}

/// This package's manifest directory, relative to `root` and normalized to forward
/// slashes — the same convention `nomos_lang_rust_cargo::metadata::Manifest_Relative_Root`
/// derives from `cargo metadata`'s own `manifest_path`, derived here instead from `cargo
/// clippy`'s `package_id`, since this crate may not depend on that one to reuse its
/// reader. `None` for a registry dependency (`package_id` prefixed `registry+`, never
/// `path+file://`) or a package outside `root`.
fn First_Party_Relative_Root(package_id: &str, root: &Path) -> Option<String>
{
    let after_scheme = package_id.strip_prefix("path+file://")?;
    let (raw_path, _version) = after_scheme.rsplit_once('#')?;
    let absolute = PathBuf::from(Windows_Drive_Path(raw_path));
    let relative = absolute.strip_prefix(root).ok()?;

    return Some(relative.to_string_lossy().replace('\\', "/"));
}

/// `cargo`'s own `path+file://` URI form prefixes a Windows drive letter with an extra
/// leading slash (`/F:/repos/...`), which [`PathBuf`] would otherwise read as a
/// Unix-rooted path through a directory literally named `F:`. Strips it only in exactly
/// that shape; a POSIX path (`/home/...`) is returned unchanged.
fn Windows_Drive_Path(raw: &str) -> &str
{
    let [b'/', drive, b':', ..] = raw.as_bytes()
    else
    {
        return raw;
    };

    if drive.is_ascii_alphabetic()
    {
        return &raw[1..];
    }

    return raw;
}

/// This package's own name, read as the last segment of its relativized manifest
/// directory — the convention every crate in this workspace already follows (its
/// directory name and its `Cargo.toml` `name` field agree), the same assumption
/// `nomos_lang_rust_cargo`'s own `Discover_Workspace` does not need to make only because
/// `cargo metadata` hands its `name` field back directly; `cargo clippy`'s own
/// `package_id` does not carry the declared name at all, only the manifest's path.
fn Package_Name(relative_root: &str) -> String
{
    return relative_root.rsplit('/').next().unwrap_or(relative_root).to_owned();
}

/// One diagnostic out of `cargo clippy`'s own top-level `"message"` object — never a
/// `note`/`help` child, which describes a diagnostic rather than being one. `None` for a
/// level this schema does not recognize, or a message with no primary span: this reader
/// cannot attribute a file or line to a diagnostic that names neither, and a diagnostic
/// nobody could report a location for is not a fact this capability's own contract
/// promises.
fn Diagnostic_Of(message: &serde_json::Value) -> Option<LintDiagnostic>
{
    let level = LintLevel::From_Label(message.get("level")?.as_str()?)?;
    let text = message.get("message")?.as_str()?.to_owned();
    let lint_id = message
        .get("code")
        .and_then(|code| return code.get("code"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    let primary = Primary_Span(message)?;
    let file = primary.get("file_name")?.as_str()?.replace('\\', "/");
    let line_number = u32::try_from(primary.get("line_start")?.as_u64()?).ok()?;

    return Some(LintDiagnostic { level, lint: lint_id, message: text, file, line: line_number });
}

/// `message`'s own primary span, the one its rendering points a reader at.
fn Primary_Span(message: &serde_json::Value) -> Option<&serde_json::Value>
{
    let spans = message.get("spans")?.as_array()?;

    return spans
        .iter()
        .find(|span| return span.get("is_primary").and_then(serde_json::Value::as_bool) == Some(true));
}

/// Refuses an empty result: `cargo clippy` reporting no first-party package at all means
/// this reader saw nothing, not that the workspace itself is empty.
fn Require_Nonempty(discovered: Vec<DiscoveredDiagnostics>) -> Result<Vec<DiscoveredDiagnostics>, ClippyError>
{
    if discovered.is_empty()
    {
        return Err(ClippyError {
            reason: "cargo clippy reported no first-party workspace member; refusing to \
                     report a clean result over an empty workspace"
                .to_owned(),
        });
    }

    return Ok(discovered);
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Registry_Package_Id_Should_Not_Resolve()
    {
        let root = Path::new("F:/repos/nomos");
        let id = "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.229";

        assert_eq!(First_Party_Relative_Root(id, root), None);
    }

    #[test]
    fn Test_A_First_Party_Package_Id_Should_Resolve_Relative_To_Root()
    {
        let root = Path::new("F:/repos/nomos");
        let id = "path+file:///F:/repos/nomos/crates/substrate/nomos-ledger#0.1.0";

        assert_eq!(
            First_Party_Relative_Root(id, root),
            Some("crates/substrate/nomos-ledger".to_owned())
        );
    }

    #[test]
    fn Test_A_Posix_Package_Id_Should_Resolve_Without_A_Drive_Letter()
    {
        let root = Path::new("/home/build/nomos");
        let id = "path+file:///home/build/nomos/crates/substrate/nomos-ledger#0.1.0";

        assert_eq!(
            First_Party_Relative_Root(id, root),
            Some("crates/substrate/nomos-ledger".to_owned())
        );
    }

    #[test]
    fn Test_Package_Name_Reads_The_Last_Path_Segment()
    {
        assert_eq!(Package_Name("crates/substrate/nomos-ledger"), "nomos-ledger");
    }

    /// A real, captured `cargo clippy --message-format=json` diagnostic, from this
    /// workspace's own output over `nomos-ledger` before this reader existed to parse it —
    /// not invented, so a change to `rustc`'s own JSON shape is caught here rather than
    /// only against a fixture written to already agree with this code.
    fn Real_Compiler_Message() -> serde_json::Value
    {
        return serde_json::json!({
            "reason": "compiler-message",
            "package_id": "path+file:///F:/repos/nomos/crates/substrate/nomos-ledger#0.1.0",
            "message": {
                "rendered": "warning: binding's name is too similar to existing binding\n",
                "$message_type": "diagnostic",
                "children": [],
                "code": { "code": "clippy::similar_names", "explanation": null },
                "level": "warning",
                "message": "binding's name is too similar to existing binding",
                "spans": [
                    {
                        "file_name": "crates\\substrate\\nomos-ledger\\src\\finish.rs",
                        "line_start": 113,
                        "line_end": 113,
                        "column_start": 9,
                        "column_end": 12,
                        "is_primary": true
                    }
                ]
            }
        });
    }

    #[test]
    fn Test_A_Real_Captured_Diagnostic_Should_Parse()
    {
        let message = Real_Compiler_Message();
        let diagnostic = Diagnostic_Of(message.get("message").expect("captured fixture has a message"))
            .expect("a real compiler-message with a primary span must parse");

        assert_eq!(diagnostic.level, LintLevel::Warning);
        assert_eq!(diagnostic.lint.as_deref(), Some("clippy::similar_names"));
        assert_eq!(diagnostic.message, "binding's name is too similar to existing binding");
        assert_eq!(diagnostic.file, "crates/substrate/nomos-ledger/src/finish.rs");
        assert_eq!(diagnostic.line, 113);
    }

    #[test]
    fn Test_A_Message_With_No_Primary_Span_Should_Not_Parse()
    {
        let message = serde_json::json!({
            "level": "help",
            "message": "for further information visit https://...",
            "code": null,
            "spans": []
        });

        assert_eq!(Diagnostic_Of(&message), None);
    }

    #[test]
    fn Test_A_Message_With_No_Lint_Code_Should_Still_Parse()
    {
        let message = serde_json::json!({
            "level": "error",
            "message": "mismatched types",
            "code": null,
            "spans": [{ "file_name": "a.rs", "line_start": 1, "is_primary": true }]
        });

        let diagnostic = Diagnostic_Of(&message).expect("a code-less diagnostic still names a level, message and span");

        assert_eq!(diagnostic.lint, None);
    }

    #[test]
    fn Test_Grouped_By_Package_Reports_A_Clean_Member_With_No_Diagnostics()
    {
        let root = Path::new("F:/repos/nomos");
        let stdout = serde_json::json!({
            "reason": "compiler-artifact",
            "package_id": "path+file:///F:/repos/nomos/crates/contracts/nomos-contracts#0.1.0",
            "target": { "kind": ["lib"] }
        })
        .to_string();

        let discovered = Grouped_By_Package(&stdout, root);

        assert_eq!(discovered.len(), 1, "{discovered:?}");
        let member = discovered.first().expect("asserted len 1 above");
        assert_eq!(member.payload.package, "nomos-contracts");
        assert!(member.payload.diagnostics.is_empty(), "an artifact with no message is a clean report, not an absent one");
    }

    #[test]
    fn Test_A_Malformed_Json_Line_Should_Be_Skipped_Rather_Than_Fail_The_Whole_Stream()
    {
        let root = Path::new("F:/repos/nomos");
        let stdout = "not json at all\n";

        assert!(Grouped_By_Package(stdout, root).is_empty());
    }

    #[test]
    fn Test_Duplicate_Diagnostics_From_Two_Target_Compiles_Should_Collapse_To_One()
    {
        let root = Path::new("F:/repos/nomos");
        let one_message = |package_id: &str| {
            return serde_json::json!({
                "reason": "compiler-message",
                "package_id": package_id,
                "message": {
                    "level": "warning",
                    "message": "unneeded return statement",
                    "code": { "code": "clippy::needless_return" },
                    "spans": [{ "file_name": "src/lib.rs", "line_start": 5, "is_primary": true }]
                }
            })
            .to_string();
        };
        let id = "path+file:///F:/repos/nomos/crates/rules/nomos-rules#0.1.0";
        let stdout = format!("{}\n{}\n", one_message(id), one_message(id));

        let discovered = Grouped_By_Package(&stdout, root);

        assert_eq!(discovered.len(), 1);
        assert_eq!(
            discovered.first().expect("asserted len 1 above").payload.diagnostics.len(),
            1,
            "the same diagnostic reported by two target compiles must collapse to one"
        );
    }
}
