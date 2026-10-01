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

/// How long `cargo clippy` may go without printing anything before it is judged stalled --
/// the bound that tells a hung tool from a working one.
///
/// `--message-format json` prints a line as each crate finishes, so a clippy that is still
/// working keeps resetting this clock however long the whole run takes. A single wall bound
/// could not tell the two apart: it measured how loaded the machine was as much as whether
/// the tool had hung. Measured on 2026-09-27 on a 32-thread machine, a cold run over this
/// whole workspace took 24.5 seconds and never went more than 2.9 seconds without a line.
/// So 600 seconds as a wall bound misjudges a working run once everything is about 24 times
/// slower than that, and as an idle bound only at about 200 times -- and a slowdown is what a
/// shared machine running several suites at once produces. A cold target directory alone did
/// not reproduce the failures that led here; `P134-A-PREDICATE-DECIDES-ON-TARGET-DIRECTORY-
/// STATE-AND-NAMES-THREE-UNRELATED-TESTS-WHEN-IT-DOES` records the runs.
const IDLE_BOUND: Duration = Duration::from_secs(600);

/// What cargo prints, once, on stderr when another cargo process holds the lock on the same
/// build directory -- after which it prints nothing until that lock is released.
///
/// Declared to the launcher as a wait rather than left to read as silence, so a clippy that is
/// queued behind a peer's build is not killed as a stall at [`IDLE_BOUND`]; [`LOCK_WAIT_BOUND`]
/// ends a wait that never does. Measured by `P169-A-CLIPPY-WAITING-ON-A-PEERS-BUILD-LOCK-IS-
/// KILLED-AS-A-STALL` with a real second cargo holding the lock: this one line, then silence
/// for exactly as long as the other process held it. Of this workspace's providers only this
/// one takes the build lock -- `cargo metadata` answered in 65 ms while it was held, `cargo deny
/// check` reads metadata, and the compiler-backed provider runs no `cargo check`.
const CARGO_LOCK_WAIT: &str = "Blocking waiting for file lock";

/// How long one wait on a peer's build lock may last before its silence counts toward
/// [`IDLE_BOUND`] again, and the run is ended as the stall a lock that never frees makes it.
///
/// Derived, not chosen. A peer holds the build directory's lock only while it builds: `cargo test`
/// releases it before running tests (measured 2026-09-27 on toolchain 1.88.0 -- a nested `cargo
/// check`, and then this provider's exact `cargo clippy` invocation, over the same target directory
/// finished in 0.2 s and 0.4 s under `cargo test`). The longest build a peer runs on this
/// workspace since `OD-GATE-035` is a cold `cargo test --workspace --no-run`, about 91 seconds
/// (a cold workspace clippy about 70). Thirty minutes admits some twenty such holds queued back
/// to back -- far more peers than this board runs at once -- and is a twelfth of [`WALL_BOUND`].
///
/// What it ends: on 2026-09-27 a nested clippy waited ninety minutes on a lock whose holder never
/// released it, holding a machine-wide lock of its own that stalled every other cargo on the
/// host. Before `P169` that wait died at [`IDLE_BOUND`]; after it, only [`WALL_BOUND`] was left.
/// Now it ends at this bound plus [`IDLE_BOUND`], reported as the tool that could not answer it
/// is. Who held that lock was not captured; the next wait to exceed this bound is the chance to.
const LOCK_WAIT_BOUND: Duration = Duration::from_secs(30 * 60);

/// How long `cargo clippy` may run in all, however much it prints: the six hours GitHub
/// Actions gives the identical gate step, which declares no `timeout-minutes` of its own.
///
/// Stated rather than measured, and deliberately not a guess at how long a cold run takes:
/// this provider runs the same invocation the gate's lint step runs, so a shorter bound here
/// would refuse an answer the gate itself would wait for. Hangs are [`IDLE_BOUND`]'s to
/// catch; this bound only ends a run that keeps printing and never finishes.
const WALL_BOUND: Duration = Duration::from_secs(6 * 60 * 60);

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
/// [`ClippyError`] if the `cargo` binary cannot be run, exits non-zero for any reason other than
/// a build that failed on errors it reported in a workspace member, is killed for exceeding
/// [`WALL_BOUND`] or for printing nothing for [`IDLE_BOUND`], or its stdout could not be read as
/// the newline-delimited JSON stream `--message-format json` promises.
///
/// # A build that failed is an answer
///
/// `cargo clippy` exits non-zero when a member does not compile and when a `deny` lint fires -- the
/// two things `OD-GATE-007` says block a merge -- and it says so in the stream: each error as a
/// `compiler-message` at level `error`, and then `build-finished` with `success: false`. Measured on
/// toolchain 1.88.0 against both. That run answered the question this provider asks; reading it as a
/// tool that could not run would drop the errors and every other diagnostic with them, which is how
/// a workspace that did not compile once read as a clean gate. So a finished, failed build is read
/// like a passing one, errors included. A run that exited non-zero any other way -- no manifest
/// (`could not find Cargo.toml`, empty stdout), a build that failed only in a dependency, a stall,
/// a timeout -- is still a failure to answer.
pub fn Discover_Workspace<Launcher: ProgramLauncher, Env: Environment>(root: &Path, launcher: &Launcher, environment: &Env) -> Result<Vec<DiscoveredDiagnostics>, ClippyError>
{
    let run = Run_Cargo_Clippy(root, launcher, environment)?;
    let absolute_root = Absolute_Path_Of(root, environment)?;
    let discovered = Grouped_By_Package(&run.stdout, &absolute_root);
    if run.build_failed && !discovered.iter().any(|member| return member.payload.diagnostics.iter().any(|diagnostic| return diagnostic.level == LintLevel::Error))
    {
        return Err(ClippyError {
            reason: format!(
                "cargo clippy's build failed and it reported no error in any workspace member, so the failure is outside the code this provider judges: {}",
                run.stderr.trim()
            ),
        });
    }

    return Require_Nonempty(discovered);
}

/// What one `cargo clippy` run left to read: its stream, its stderr, and whether its build
/// finished and failed.
struct ClippyRun
{
    stdout: String,
    stderr: String,
    build_failed: bool,
}

fn Run_Cargo_Clippy<Launcher: ProgramLauncher, Env: Environment>(root: &Path, launcher: &Launcher, environment: &Env) -> Result<ClippyRun, ClippyError>
{
    let command = Cargo_Clippy_Command(root, environment);
    let output = launcher.Run(&command).map_err(|error| ClippyError {
        reason: format!("cargo clippy could not be run: {error}"),
    })?;

    let build_failed = matches!(output.outcome, ExitOutcome::Exited { code } if code != 0) && Finished_A_Failed_Build(&output.stdout);
    if !build_failed
    {
        Require_Clean_Exit(&output.outcome, &output.stderr)?;
    }

    return Ok(ClippyRun { stdout: output.stdout, stderr: output.stderr, build_failed });
}

/// Whether the stream ends a build cargo finished and reports as failed: a `build-finished` message
/// with `success: false`.
fn Finished_A_Failed_Build(stdout: &str) -> bool
{
    return stdout.lines().filter_map(|line| return serde_json::from_str::<serde_json::Value>(line).ok()).any(|value| {
        return value.get("reason").and_then(serde_json::Value::as_str) == Some("build-finished")
            && value.get("success").and_then(serde_json::Value::as_bool) == Some(false);
    });
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
        WALL_BOUND,
    )
    .With_Idle_Timeout(IDLE_BOUND)
    .While_Waiting_Prints(CARGO_LOCK_WAIT, LOCK_WAIT_BOUND);
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
            reason: format!("cargo clippy was still running after {WALL_BOUND:?} and was killed"),
        }),
        ExitOutcome::Stalled { idle_elapsed } => Err(ClippyError { reason: Stalled_Reason(*idle_elapsed, stderr) }),
        ExitOutcome::Terminated => Err(ClippyError {
            reason: "cargo clippy was terminated before it could finish".to_owned(),
        }),
    };
}

/// Why a stalled clippy stalled, in the words a person needs next.
///
/// A stall whose last line was cargo's lock wait is a wait that outlasted [`LOCK_WAIT_BOUND`]: a
/// lock nothing released. The ninety-minute case this bound came from ended without anyone learning
/// which process held that lock, so the reason says to find out while the holder is still alive, and
/// where the lock is.
fn Stalled_Reason(idle_elapsed: Duration, stderr: &str) -> String
{
    let waited_on_a_lock = stderr.lines().rev().find(|line| return !line.trim().is_empty()).is_some_and(|line| return line.contains(CARGO_LOCK_WAIT));
    if waited_on_a_lock
    {
        return format!(
            "cargo clippy waited on another cargo's build lock for longer than {LOCK_WAIT_BOUND:?} and then printed nothing for {idle_elapsed:?}, so it was ended as stalled: \
             a lock that is not released is held by a process that is not finishing -- find which one holds the target directory's `.cargo-lock` \
             before it exits, and record it on the board"
        );
    }

    return format!("cargo clippy produced no output for {idle_elapsed:?} and was judged stalled");
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

    /// The provider's own command: a hung clippy is judged stalled at [`IDLE_BOUND`], a clippy
    /// queued behind another cargo's lock on the build directory is declared waiting rather than
    /// silent, and every run is capped at [`WALL_BOUND`]. The launcher's own tests prove what a
    /// declared wait does; this proves the provider asks for it.
    #[test]
    fn Test_The_Clippy_Command_Should_Declare_Cargos_Lock_Wait_And_Both_Bounds()
    {
        let command = Cargo_Clippy_Command(Path::new("."), &StdEnvironment);

        assert_eq!(command.idle_timeout, IDLE_BOUND);
        assert_eq!(command.timeout, WALL_BOUND);
        assert_eq!(command.waiting_line.as_deref(), Some(CARGO_LOCK_WAIT));
        assert_eq!(command.waiting_timeout, LOCK_WAIT_BOUND);
        assert!(LOCK_WAIT_BOUND < WALL_BOUND, "a wait bounded only by the wall bound is the hang this bound ends");
    }

    /// A stall whose last line is cargo's lock wait says so, names the lock, and says to find its
    /// holder; any other stall is reported as the plain silence it is.
    #[test]
    fn Test_A_Stall_On_A_Lock_Should_Say_Whose_Lock_To_Look_For()
    {
        let on_a_lock = Stalled_Reason(IDLE_BOUND, "    Checking a
    Blocking waiting for file lock on build directory
");
        let silent = Stalled_Reason(IDLE_BOUND, "    Checking a
");

        assert!(on_a_lock.contains("build lock") && on_a_lock.contains(".cargo-lock"), "{on_a_lock}");
        assert!(!silent.contains(".cargo-lock"), "{silent}");
    }

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

    /// A launcher whose `cargo clippy` exited 101 with `stdout` and `stderr` -- a failed build, a
    /// missing manifest, or anything else a non-zero exit carries.
    struct FailedLauncher
    {
        stdout: String,
        stderr: String,
    }

    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for FailedLauncher
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl ProgramLauncher for FailedLauncher
    {
        fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
        {
            return Ok(ProgramOutput { outcome: ExitOutcome::Exited { code: 101 }, stdout: self.stdout.clone(), stderr: self.stderr.clone() });
        }
    }

    /// The stream a build that failed on one error in `package` prints, in the shape measured on
    /// 1.88.0: the error as a `compiler-message`, cargo's own span-less "aborting" message, and
    /// `build-finished` with `success: false`.
    fn Failed_Build_Stream(package: &Path) -> String
    {
        let error = serde_json::json!({ "reason": "compiler-message", "package_id": Package_Id_Uri(package), "message": {
            "level": "error", "message": "mismatched types", "code": { "code": "E0308" },
            "spans": [{ "file_name": "src/lib.rs", "line_start": 1, "is_primary": true }] } });
        let aborting = serde_json::json!({ "reason": "compiler-message", "package_id": Package_Id_Uri(package), "message": {
            "level": "error", "message": "aborting due to 1 previous error", "code": null, "spans": [] } });
        let finished = serde_json::json!({ "reason": "build-finished", "success": false });
        return format!("{error}\n{aborting}\n{finished}\n");
    }

    /// A member that does not compile is an answer: its error is read, at level `error`, rather than
    /// the run being refused and every diagnostic dropped with it.
    #[test]
    fn Test_A_Build_That_Failed_In_A_Member_Should_Be_Read_With_Its_Error()
    {
        let root = std::env::temp_dir().join("nomos-lang-rust-clippy-failed-build-fixture");
        let member = root.join("crates").join("broken");
        let launcher = FailedLauncher { stdout: Failed_Build_Stream(&member), stderr: "error: could not compile `broken`".to_owned() };

        let discovered = Discover_Workspace(&root, &launcher, &StdEnvironment).expect("a finished, failed build is an answer");

        let diagnostics = &discovered.first().expect("the failing member").payload.diagnostics;
        assert_eq!(diagnostics.len(), 1, "the span-less aborting message is not a diagnostic: {diagnostics:?}");
        assert_eq!(diagnostics.first().map(|diagnostic| return diagnostic.level), Some(LintLevel::Error));
    }

    /// Exit 101 with no stream is not a failed build: a tree with no manifest, among others, stays a
    /// failure to answer, and so does a failed build whose only errors are in a dependency.
    #[test]
    fn Test_A_Failure_That_Is_Not_A_Members_Error_Should_Still_Be_Refused()
    {
        let root = std::env::temp_dir().join("nomos-lang-rust-clippy-no-manifest-fixture");
        let no_manifest = FailedLauncher { stdout: String::new(), stderr: "error: could not find `Cargo.toml` in `x` or any parent directory".to_owned() };
        let dependency = FailedLauncher { stdout: Failed_Build_Stream(&std::env::temp_dir().join("somewhere-else")), stderr: "error: could not compile `dep`".to_owned() };

        let refused = Discover_Workspace(&root, &no_manifest, &StdEnvironment).expect_err("no manifest");
        let outside = Discover_Workspace(&root, &dependency, &StdEnvironment).expect_err("an error in no workspace member");

        assert!(refused.reason.contains("exit 101"), "{}", refused.reason);
        assert!(outside.reason.contains("outside the code this provider judges"), "{}", outside.reason);
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
