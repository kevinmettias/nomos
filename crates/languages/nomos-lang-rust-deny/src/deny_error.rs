//! Running `cargo deny --format json check bans licenses sources` and reading the
//! workspace's own resolved-graph verdict out of the newline-delimited JSON stream it
//! prints, one JSON object per line — verified directly against this workspace's own
//! `cargo deny` output before this reader was written, not assumed from the format's name
//! alone.

use nomos_cap_dependency_policy::{PolicySeverity, PolicyViolation};
use nomos_platform::{Command, Environment, ExitOutcome, ProgramLauncher};
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
pub fn Discover_Workspace<Launcher: ProgramLauncher, Env: Environment>(root: &Path, launcher: &Launcher, environment: &Env) -> Result<Vec<PolicyViolation>, DenyError>
{
    let config = Required_Deny_Config(root)?;
    let stream = Run_Cargo_Deny(root, &config, launcher, environment)?;

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
            reason: format!("no deny.toml at {} -- refusing rather than letting cargo deny's own upward search silently pick up an ancestor's config", config.display()),
        });
    }

    return Ok(config);
}

fn Run_Cargo_Deny<Launcher: ProgramLauncher, Env: Environment>(root: &Path, config: &Path, launcher: &Launcher, environment: &Env) -> Result<String, DenyError>
{
    let command = Cargo_Deny_Command(root, config, environment);
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

/// `config` is passed explicitly rather than left for `cargo deny` to discover on its own
/// -- [`Required_Deny_Config`]'s own doc explains why that discovery is a second escape
/// this provider must not leave open.
fn Cargo_Deny_Command<Env: Environment>(root: &Path, config: &Path, environment: &Env) -> Command
{
    let cargo = Cargo_Program(environment);
    let mut command = Command::From_String_Arguments(
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
/// emptiness guard in [`Run_Cargo_Deny`] catches only a *silent* failure; and
/// [`Violations_Of`] skips every line it cannot read, equally deliberately, because real
/// runs interleave noise. Each is defensible alone, and together they made "checked
/// everything, found nothing" and "failed, and nothing readable came back" the same value
/// -- `Ok(vec![])`, which `Materialize_Workspace` publishes as a clean dependency-policy
/// fact. A supply-chain provider reporting clean because it could not read its own tool is
/// exactly the false-coverage shape `OD-COMPLETENESS-001` exists to refuse. `P81`.
///
/// The summary is what is judged rather than the exit code, because the exit code genuinely
/// cannot carry this: `cargo deny` exits non-zero whenever a diagnostic reaches `deny` --
/// this workspace's own real run exits 4 while succeeding completely.
fn Require_Summarized(stream: &str, command: &Command) -> Result<(), DenyError>
{
    let Some(summary) = Summarized_Fields(stream)
    else
    {
        let said = Logged_Error(stream).unwrap_or_else(|| return format!("stderr: {stream}"));

        return Err(DenyError {
            reason: format!("cargo deny wrote no summary line, so it did not finish its checks and an empty violation list over it is not a clean result -- {said}"),
        });
    };

    let unaccounted = Unaccounted_Checks(&summary, command);
    if !unaccounted.is_empty()
    {
        return Err(DenyError {
            reason: format!("cargo deny's own summary accounts for none of {unaccounted:?}, so those checks did not finish and their silence is not evidence of a clean result; stderr: {stream}"),
        });
    }

    return Ok(());
}

/// The one `{"type":"summary"}` object a completed `cargo deny` run ends its stream with,
/// or `None` when no line in the stream is one.
fn Summarized_Fields(stream: &str) -> Option<serde_json::Value>
{
    return stream
        .lines()
        .filter_map(|line| return serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|value| return value.get("type").and_then(serde_json::Value::as_str) == Some("summary"));
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

/// The checks `command` asked for that `summary`'s own fields do not account for -- the
/// checks whose silence over this stream is therefore not evidence of anything.
fn Unaccounted_Checks(summary: &serde_json::Value, command: &Command) -> Vec<String>
{
    return Requested_Checks(command)
        .into_iter()
        .filter(|check| return summary.get("fields").and_then(|fields| return fields.get(check)).is_none())
        .collect();
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
    let target = Target_Of(fields);

    return Some(PolicyViolation { severity, code, target, message });
}

/// What a diagnostic is about, from `cargo deny`'s own `graphs` array.
///
/// Each entry is a dependency-graph root, and its `name` is the package the diagnostic
/// concerns. The first is taken: a diagnostic naming several roots is reporting one
/// violation reachable by several paths, not several violations, and the package is the
/// same one at the top of each.
///
/// `None` when there is no graph, which is a real answer rather than a gap -- an
/// unencountered license concerns no package. Read rather than invented: this was the field
/// the parser dropped, and dropping it made twelve violations about twelve different crates
/// into twelve byte-identical facts.
fn Target_Of(fields: &serde_json::Value) -> Option<String>
{
    // `graphs[0].Krate.name`, not `graphs[0].name`. Every fixture in this file carries an
    // empty `graphs`, so only real `cargo deny` output shows the nesting -- measured
    // 2026-09-14 against cargo-deny 0.20.2, whose source-not-allowed diagnostics wrap each
    // graph root in a `Krate` object.
    let root = fields.get("graphs")?.as_array()?.first()?;
    let name = root.get("Krate")?.get("name")?.as_str()?;

    return (!name.is_empty()).then(|| return name.to_owned());
}

#[cfg(test)]
#[path = "reading/tests.rs"]
mod tests;
