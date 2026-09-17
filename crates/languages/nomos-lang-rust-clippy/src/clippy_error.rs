//! Running `cargo clippy --message-format=json` and reading a workspace's own first-party
//! lint diagnostics out of the newline-delimited JSON stream it prints, one JSON object per
//! line — a real, structural difference from `cargo metadata --format-version 1`'s one
//! document, verified directly against this workspace's own `cargo clippy` output before
//! this reader was written, not assumed from the format's name alone.

use nomos_cap_lint::{DiagnosticsPayload, LintDiagnostic, LintLevel};
use nomos_platform::{Command, Environment, ExitOutcome, ProgramLauncher};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[path = "reading/discovered_diagnostics.rs"]
mod discovered_diagnostics;
#[cfg(test)]
mod tests;

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
pub fn Discover_Workspace<Launcher: ProgramLauncher, Env: Environment>(root: &Path, launcher: &Launcher, environment: &Env) -> Result<Vec<DiscoveredDiagnostics>, ClippyError>
{
    let stdout = Run_Cargo_Clippy(root, launcher, environment)?;
    let absolute_root = Absolute_Path_Of(root, environment)?;
    let discovered = Grouped_By_Package(&stdout, &absolute_root);

    return Require_Nonempty(discovered);
}

fn Run_Cargo_Clippy<Launcher: ProgramLauncher, Env: Environment>(root: &Path, launcher: &Launcher, environment: &Env) -> Result<String, ClippyError>
{
    let command = Cargo_Clippy_Command(root, environment);
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
fn Cargo_Clippy_Command<Env: Environment>(root: &Path, environment: &Env) -> Command
{
    let cargo = Cargo_Program(environment);
    let mut command = Command::From_String_Arguments(
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

/// The program name `cargo` is invoked by, from the environment rather than from this
/// process's own ambient state.
///
/// `CARGO` is what a cargo-invoked build sets to the exact toolchain binary running, and a
/// provider handed an injected launcher must not then reach around it for the program that
/// launcher will run -- a fake launcher receives the command already built, so no test could
/// state which cargo it names. `P87`/`OD-HOST-001`: the port comes from the composition root.
fn Cargo_Program<Env: Environment>(environment: &Env) -> String
{
    return environment
        .Variable("CARGO")
        .and_then(|value| return value.into_string().ok())
        .unwrap_or_else(|| return "cargo".to_owned());
}

/// Refuses every outcome a launched process can report other than a clean, zero exit —
/// the same shape `nomos_lang_rust_cargo`'s own `Require_Clean_Exit` uses for the identical
/// reason: a real compile error, not merely a lint warning, is the only thing that makes
/// `cargo clippy` exit non-zero here, since this workspace's own gate step carries no
/// `-D warnings`.
fn Require_Clean_Exit(outcome: &ExitOutcome, stderr: &str) -> Result<(), ClippyError>
{
    return match outcome
    {
        ExitOutcome::Exited { code: 0 } => Ok(()),
        ExitOutcome::Exited { code } => Err(ClippyError {
            reason: format!("cargo clippy failed (exit {code}): {stderr}"),
        }),
        ExitOutcome::TimedOut => Err(ClippyError {
            reason: format!("cargo clippy was still running after {TIMEOUT:?} and was killed"),
        }),
        ExitOutcome::Stalled { idle_elapsed } => Err(ClippyError {
            reason: format!("cargo clippy produced no output for {idle_elapsed:?} and was judged stalled"),
        }),
        ExitOutcome::Terminated => Err(ClippyError {
            reason: "cargo clippy was terminated before it could finish".to_owned(),
        }),
    };
}

/// `root` made absolute against the real process working directory -- not
/// `std::fs::canonicalize`, whose Windows implementation returns a `\\?\`-prefixed
/// verbatim path that a `cargo clippy`-reported `package_id` never carries, which would
/// silently break every prefix match in [`First_Party_Relative_Root`] below (the exact
/// footgun `nomos_lang_rust_compiler::reading::Load_Crate`'s own doc already names, for
/// the identical reason: a *prefix* comparison, unlike a plain equality check, cannot
/// tolerate one side being canonicalized and the other not). A relative root -- `nomos
/// check`'s own CLI default is `.` -- must resolve to the same real directory `cargo
/// clippy` itself ran in, or every first-party package it reports would fail to
/// relativize against it and be excluded as if it were external.
fn Absolute_Path_Of<Env: Environment>(root: &Path, environment: &Env) -> Result<PathBuf, ClippyError>
{
    let current_dir = environment.Working_Directory().map_err(|error| ClippyError {
        reason: format!("the current directory could not be read: {error}"),
    })?;

    return Ok(current_dir.join(root));
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

/// This package's manifest directory, relative to `root` and normalized to forward
/// slashes — the same convention `nomos_lang_rust_cargo::metadata::Manifest_Relative_Root`
/// derives from `cargo metadata`'s own `manifest_path`, derived here instead from `cargo
/// clippy`'s `package_id`, since this crate may not depend on that one to reuse its
/// reader.
///
/// `None` for a registry dependency (`package_id` prefixed `registry+`, never
/// `path+file://`) and, just as importantly, for a path package `root` cannot relativize
/// against: `P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT` measured directly that folding
/// such a package in under its own absolute path instead (as this function used to) is
/// exactly how a `cargo clippy --workspace` escape past a nested root -- reporting on the
/// enclosing workspace instead of refusing -- went unnoticed: every escaped package still
/// got a manifest-relative-looking string, just one that was actually somebody else's
/// absolute path. `root` is expected to already be absolute (`Discover_Workspace`'s own
/// [`Absolute_Path_Of`] guarantees this for every real caller); a package genuinely outside it
/// is excluded, not included under a different key.
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

/// Refuses an empty result: `cargo clippy` reporting no first-party package at all means
/// this reader saw nothing, not that the workspace itself is empty.
fn Require_Nonempty(discovered: Vec<DiscoveredDiagnostics>) -> Result<Vec<DiscoveredDiagnostics>, ClippyError>
{
    if discovered.is_empty()
    {
        return Err(ClippyError {
            reason: "cargo clippy reported no first-party workspace member; refusing to report a clean result over an empty workspace".to_owned(),
        });
    }

    return Ok(discovered);
}

#[cfg(test)]
mod local_tests
{
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
    use nomos_platform_std::StdEnvironment;
    use super::*;
    use nomos_platform::ProgramOutput;

    /// A launcher that hands `Discover_Workspace` a fixed JSON-lines stream instead of
    /// running a real `cargo clippy` — the boundary this crate's own module doc names as
    /// the one place a caller substitutes a real subprocess.
    struct FakeLauncher
    {
        stdout: String,
    }

    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for FakeLauncher
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl ProgramLauncher for FakeLauncher
    {
        fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
        {
            return Ok(ProgramOutput {
                outcome: ExitOutcome::Exited { code: 0 },
                stdout: self.stdout.clone(),
                stderr: String::new(),
            });
        }
    }

    /// A `path+file://` package id `cargo clippy` could plausibly report for `path` --
    /// built from a real [`PathBuf`] rather than a hand-typed literal so a fixture can
    /// name a real, absolute directory (a real temp directory, or the test's own real
    /// current directory) without also hand-encoding its drive letter and separators.
    fn Package_Id_Uri(path: &Path) -> String
    {
        let forward = path.to_string_lossy().replace('\\', "/");
        let rooted = if forward.starts_with('/') { forward } else { format!("/{forward}") };

        return format!("path+file://{rooted}#0.1.0");
    }

    /// The root is a real temp directory rather than a hand-typed literal, and the package
    /// id is built from it through [`Package_Id_Uri`], because this test needs its root to
    /// be *absolute* and a drive-lettered literal is only absolute on one platform.
    ///
    /// It used to root itself at `F:/repos/nomos`. Windows reads that as absolute and the
    /// prefix match succeeded; Linux reads it as relative, and
    /// [`Discover_Workspace`]'s own resolution of a relative root against the real current
    /// directory then put the root somewhere the hand-typed id could never sit under, so it
    /// found no first-party member and refused. The test passed on one developer's machine
    /// and failed in CI on ubuntu alone. `P79`.
    #[test]
    fn Test_Discover_Workspace_Should_Read_A_First_Party_Package_From_The_Json_Stream()
    {
        let root = std::env::temp_dir().join("nomos-lang-rust-clippy-first-party-fixture");
        let member = root.join("crates").join("contracts").join("nomos-contracts");
        let stdout = serde_json::json!({
            "reason": "compiler-artifact",
            "package_id": Package_Id_Uri(&member),
            "target": { "kind": ["lib"] }
        })
        .to_string();
        let launcher = FakeLauncher { stdout };

        let discovered = Discover_Workspace(&root, &launcher, &StdEnvironment).expect("the fake launcher reports one package");

        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered.first().expect("one package").payload.package, "nomos-contracts");
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_An_Empty_Stream()
    {
        let root = Path::new("F:/repos/nomos");
        let launcher = FakeLauncher { stdout: String::new() };

        let error = Discover_Workspace(root, &launcher, &StdEnvironment).expect_err("an empty stream names no first-party package");

        assert!(error.reason.contains("no first-party workspace member"), "{}", error.reason);
    }

    /// A relative root -- `nomos check`'s own CLI default is `.` -- must resolve to the
    /// real directory the fake launcher's own report is genuinely nested under, not fail
    /// to relativize and be excluded. Built from the test's own real current directory
    /// (`Discover_Workspace`'s [`Absolute_Path_Of`] resolves `.` against exactly that), rather
    /// than a hardcoded absolute path with no real relationship to it: before `Absolute_Path_Of`
    /// existed, this test only passed because of the very fallback-to-inclusion bug
    /// `P68-SUBPROCESS-PROVIDERS-ESCAPE-A-NESTED-ROOT` fixes, not because the package was
    /// ever really found under `.`.
    #[test]
    fn Test_Discover_Workspace_Should_Not_Refuse_Under_A_Relative_Root()
    {
        let root = Path::new(".");
        let current_dir = std::env::current_dir().expect("a real test process has a real current directory");
        let member = current_dir.join("nomos-lang-rust-clippy");
        let stdout = serde_json::json!({
            "reason": "compiler-artifact",
            "package_id": Package_Id_Uri(&member),
            "target": { "kind": ["lib"] }
        })
        .to_string();
        let launcher = FakeLauncher { stdout };

        let discovered = Discover_Workspace(root, &launcher, &StdEnvironment)
            .expect("a relative root, once resolved against the real current directory, must still find a package genuinely under it");

        assert_eq!(discovered.len(), 1);
    }

    /// The regression this whole item exists to close, measured directly while building
    /// `tests/integration/fixtures/third-party/hex-0.4.3`: pointed at a root, `cargo
    /// clippy --workspace` can report a package that lives *outside* it (the escape past a
    /// nested root with no manifest of its own). Before this fix, `First_Party_Relative_
    /// Root`'s `unwrap_or` fallback folded such a package in under its own absolute path
    /// instead of excluding it, which is exactly how 174 unrelated findings about the
    /// whole enclosing repository were silently attributed to a two-file fixture. With
    /// only one (excluded) package reported, this also exercises `Require_Nonempty`'s own
    /// honest refusal rather than a clean but empty result.
    #[test]
    fn Test_Discover_Workspace_Should_Refuse_Rather_Than_Include_A_Package_Reported_Outside_Root()
    {
        let root = std::env::temp_dir().join("nomos-lang-rust-clippy-p68-root-fixture");
        let escaped = std::env::temp_dir().join("nomos-lang-rust-clippy-p68-outside-fixture").join("elsewhere");
        let stdout = serde_json::json!({
            "reason": "compiler-artifact",
            "package_id": Package_Id_Uri(&escaped),
            "target": { "kind": ["lib"] }
        })
        .to_string();
        let launcher = FakeLauncher { stdout };

        let error = Discover_Workspace(&root, &launcher, &StdEnvironment)
            .expect_err("a package cargo clippy reports outside the judged root must be excluded, leaving nothing for a real, honest Require_Nonempty refusal to report instead of a clean but empty result");

        assert!(error.reason.contains("no first-party workspace member"), "{}", error.reason);
    }
}
