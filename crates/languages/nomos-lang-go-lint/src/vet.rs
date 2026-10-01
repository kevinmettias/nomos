//! Running `go vet -json ./...` in one module and reading what it prints.
//!
//! Measured against Go 1.26.5 before this was written. `-json` prints a stream of JSON values on
//! stdout, one per package vetted: `{}` for a package with no diagnostic, and otherwise
//! `{"<import path>": {"<analyzer>": [{"posn": "<file>:<line>:<col>", "message": ...}, ...]}}`, with
//! `end` and `suggested_fixes` beside each diagnostic when the analyzer gives them. It exits 0
//! whether or not it found anything. A package that does not type-check prints no JSON; `go vet`
//! writes the compiler's error to stderr and exits 1.
//!
//! So the exit code alone cannot tell a finding from a clean run, and a clean run cannot be read off
//! an empty stdout. Exit 0 with a stream that reads is an answer, possibly with nothing in it; exit 1
//! is a failure, carrying stderr's words; anything else that `-json` prints is refused. `go help vet`
//! promises no stability for this form. It is the analysis framework's own machine output rather
//! than text written for a person, which is why it is read at all, and it is read in exactly this
//! shape so that a changed shape is refused rather than misread.

use crate::VetFailure;
use nomos_cap_lint::{LintDiagnostic, LintLevel};
use nomos_platform::{Command, Environment, ExitOutcome, ProgramLauncher};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Vetting a large module type-checks every package in it; this is headroom for that, not the
/// expected case.
const TIMEOUT: Duration = Duration::from_secs(1800);

/// Why one module has no diagnostics to report.
#[derive(Debug)]
pub(crate) struct VetError
{
    pub(crate) failure: VetFailure,
    pub(crate) reason: String,
}

/// Every diagnostic `go vet` reports for the module at `module` (an absolute directory), with
/// each file made relative to `root` (absolute), sorted and without duplicates.
pub(crate) fn Vet_Module<Launcher: ProgramLauncher, Env: Environment>(
    root: &Path,
    module: &Path,
    launcher: &Launcher,
    environment: &Env,
) -> Result<Vec<LintDiagnostic>, VetError>
{
    let mut command = Command::From_String_Arguments(vec![Go_Program(environment), "vet".to_owned(), "-json".to_owned(), "./...".to_owned()], TIMEOUT);
    command.working_directory = Some(module.to_path_buf());
    let output = launcher.Run(&command).map_err(|error| return Error(VetFailure::GoUnavailable, &format!("go could not be started: {error}")))?;

    match output.outcome
    {
        ExitOutcome::Exited { code: 0 } => {}
        ExitOutcome::Exited { code } =>
        {
            let said = output.stderr.trim();
            let failure = if said.contains("cannot find GOROOT") { VetFailure::GoUnavailable } else { VetFailure::Failed };
            return Err(Error(failure, &format!("go vet exited {code}: {said}")));
        }
        ExitOutcome::TimedOut | ExitOutcome::Stalled { .. } | ExitOutcome::Terminated =>
        {
            return Err(Error(VetFailure::DidNotFinish, &format!("go vet did not finish: {:?}", output.outcome)));
        }
    }

    let mut diagnostics = Diagnostics_Of(&output.stdout, &Forward_Slashed(&root.to_string_lossy()))?;
    diagnostics.sort_by(|left, right| return (&left.file, left.line, &left.lint, &left.message).cmp(&(&right.file, right.line, &right.lint, &right.message)));
    diagnostics.dedup();
    return Ok(diagnostics);
}

/// The program `go` is run as: `$GOROOT/bin/go` when the environment names a root, which is how a
/// host with more than one Go on its path says which toolchain it means, and `go` from the path
/// otherwise.
fn Go_Program<Env: Environment>(environment: &Env) -> String
{
    let Some(root) = environment.Variable("GOROOT").and_then(|value| return value.into_string().ok()).filter(|root| return !root.is_empty())
    else
    {
        return "go".to_owned();
    };
    let program = if cfg!(windows) { "go.exe" } else { "go" };
    return PathBuf::from(root).join("bin").join(program).to_string_lossy().into_owned();
}

/// Every diagnostic in `-json`'s stream, or a refusal naming what did not read.
fn Diagnostics_Of(stdout: &str, root: &str) -> Result<Vec<LintDiagnostic>, VetError>
{
    let mut diagnostics = Vec::new();
    for value in serde_json::Deserializer::from_str(stdout).into_iter::<serde_json::Value>()
    {
        let value = value.map_err(|error| return Unreadable(&format!("the stream is not JSON: {error}")))?;
        diagnostics.extend(Package_Diagnostics(&value, root)?);
    }

    return Ok(diagnostics);
}

/// Every diagnostic one value of the stream carries: an object of packages, each an object of
/// analyzers.
fn Package_Diagnostics(value: &serde_json::Value, root: &str) -> Result<Vec<LintDiagnostic>, VetError>
{
    let packages = value.as_object().ok_or_else(|| return Unreadable(&format!("a value that is not an object: {value}")))?;
    let mut diagnostics = Vec::new();
    for (package, analyzers) in packages
    {
        let analyzers = analyzers.as_object().ok_or_else(|| return Unreadable(&format!("package {package} holds no analyzer object")))?;
        for (analyzer, reports) in analyzers
        {
            diagnostics.extend(Analyzer_Diagnostics(analyzer, reports, root)?);
        }
    }

    return Ok(diagnostics);
}

/// One analyzer's list of reports, each read as a diagnostic.
fn Analyzer_Diagnostics(analyzer: &str, reports: &serde_json::Value, root: &str) -> Result<Vec<LintDiagnostic>, VetError>
{
    let reports = reports.as_array().ok_or_else(|| return Unreadable(&format!("analyzer {analyzer} holds no list")))?;
    return reports.iter().map(|report| return Diagnostic_Of(analyzer, report, root)).collect();
}

fn Diagnostic_Of(analyzer: &str, report: &serde_json::Value, root: &str) -> Result<LintDiagnostic, VetError>
{
    let position = report.get("posn").and_then(serde_json::Value::as_str).ok_or_else(|| return Unreadable(&format!("a {analyzer} report has no posn: {report}")))?;
    let message = report.get("message").and_then(serde_json::Value::as_str).ok_or_else(|| return Unreadable(&format!("a {analyzer} report has no message: {report}")))?;
    let (file, line) = Position_Of(position).ok_or_else(|| return Unreadable(&format!("a posn that is not file:line:column: {position}")))?;

    return Ok(LintDiagnostic {
        level: LintLevel::Warning,
        lint: Some(format!("vet::{analyzer}")),
        message: message.split_whitespace().collect::<Vec<&str>>().join(" "),
        file: Relative_To(&Forward_Slashed(file), root),
        line,
    });
}

/// The file and line of a `file:line:column` position. Split from the right, because a Windows path
/// holds a colon of its own.
fn Position_Of(position: &str) -> Option<(&str, u32)>
{
    let mut parts = position.rsplitn(3, ':');
    let _column: u32 = parts.next()?.parse().ok()?;
    let line = parts.next()?.parse().ok()?;
    let file = parts.next()?;
    return Some((file, line));
}

/// `path` relative to `root` when it lies beneath it -- without regard to ASCII case on Windows,
/// where one directory has more than one spelling -- and as given otherwise.
fn Relative_To(path: &str, root: &str) -> String
{
    let root = root.trim_end_matches('/');
    let beneath = path.get(..root.len()).zip(path.get(root.len()..)).and_then(|(head, rest)| {
        let same = if cfg!(windows) { head.eq_ignore_ascii_case(root) } else { head == root };
        return same.then_some(rest)?.strip_prefix('/');
    });

    return beneath.unwrap_or(path).to_owned();
}

/// `path` with backslashes turned forward and every `.` component dropped.
fn Forward_Slashed(path: &str) -> String
{
    return path.replace('\\', "/").split('/').filter(|component| return *component != ".").collect::<Vec<&str>>().join("/");
}

fn Unreadable(what: &str) -> VetError
{
    return Error(VetFailure::Unreadable, &format!("go vet -json printed something this reader does not know: {what}"));
}

fn Error(failure: VetFailure, reason: &str) -> VetError
{
    return VetError { failure, reason: reason.to_owned() };
}

#[cfg(test)]
mod tests;
