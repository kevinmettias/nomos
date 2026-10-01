//! Running the helper in one module and reading what it prints.
//!
//! Measured against Go 1.26.5 before this was written. The helper prints one JSON object per line,
//! each with a `kind`: `checked` names a file of a package that type-checked, `value` is one value
//! assigned to the blank identifier, and `failed` is a package that did not load or type-check, with
//! its files and the toolchain's words. It exits 0 once `go list` has answered for the module,
//! whatever that answer said about each package, and 1 -- with `go list`'s own words on stderr --
//! when it did not; `go run` exits 1 too when the helper does not build. A value always lies in a
//! file the same answer names checked, since only a package that type-checked is walked.
//!
//! So exit 0 with lines that read is an answer, exit 1 is a failure carrying stderr's words, and a
//! line of any other shape is refused rather than guessed at.

use crate::{TypesFailure, TypesPorts};
use nomos_cap_go_types::DiscardedValue;
use nomos_platform::{Command, Environment, ExitOutcome, FileSystem, ProgramLauncher};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Typing a large module compiles export data for every package it depends on; this is headroom
/// for that, not the expected case.
const TIMEOUT: Duration = Duration::from_secs(1800);

/// Why one module has no answer.
#[derive(Debug)]
pub(crate) struct TypesError
{
    pub(crate) failure: TypesFailure,
    pub(crate) reason: String,
}

impl TypesError
{
    pub(crate) fn New(failure: TypesFailure, reason: &str) -> Self
    {
        return Self { failure, reason: reason.to_owned() };
    }
}

/// What the helper answered for one module.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ModuleAnswer
{
    /// Each file of a package that type-checked, relative to the root, to the values it discards.
    pub(crate) checked: BTreeMap<String, Vec<DiscardedValue>>,
    /// Each package that did not load or type-check.
    pub(crate) failed: Vec<FailedPackage>,
}

/// A package the helper could not type-check.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FailedPackage
{
    pub(crate) package: String,
    /// Its sources, relative to the root.
    pub(crate) files: Vec<String>,
    pub(crate) reason: String,
}

/// Runs the helper in `helper` over the module at `module`, with every file made relative to
/// `root`; all three are absolute directories.
pub(crate) fn Type_Module<Launcher: ProgramLauncher, Fs: FileSystem, Env: Environment>(
    helper: &Path,
    root: &Path,
    module: &Path,
    ports: &TypesPorts<'_, Launcher, Fs, Env>,
) -> Result<ModuleAnswer, TypesError>
{
    let go = Go_Program(ports.environment);
    let arguments = vec![go.clone(), "run".to_owned(), ".".to_owned(), go, module.to_string_lossy().into_owned()];
    let mut command = Command::From_String_Arguments(arguments, TIMEOUT);
    command.working_directory = Some(helper.to_path_buf());
    let output = ports.launcher.Run(&command).map_err(|error| return TypesError::New(TypesFailure::GoUnavailable, &format!("go could not be started: {error}")))?;

    match output.outcome
    {
        ExitOutcome::Exited { code: 0 } => {}
        ExitOutcome::Exited { code } =>
        {
            let said = output.stderr.trim();
            let failure = if said.contains("cannot find GOROOT") { TypesFailure::GoUnavailable } else { TypesFailure::Failed };
            return Err(TypesError::New(failure, &format!("the helper exited {code}: {said}")));
        }
        ExitOutcome::TimedOut | ExitOutcome::Stalled { .. } | ExitOutcome::Terminated =>
        {
            return Err(TypesError::New(TypesFailure::DidNotFinish, &format!("the helper did not finish: {:?}", output.outcome)));
        }
    }

    return Answer_Of(&output.stdout, &Forward_Slashed(&root.to_string_lossy()));
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

/// The module's answer in the helper's lines, or a refusal naming the line that did not read.
fn Answer_Of(stdout: &str, root: &str) -> Result<ModuleAnswer, TypesError>
{
    let mut answer = ModuleAnswer::default();
    let mut values = Vec::new();
    for line in stdout.lines().filter(|line| return !line.trim().is_empty())
    {
        let object: serde_json::Value = serde_json::from_str(line).map_err(|error| return Unreadable(&format!("a line that is not JSON ({error}): {line}")))?;
        match object.get("kind").and_then(serde_json::Value::as_str)
        {
            Some("checked") =>
            {
                answer.checked.entry(Relative_To(&Forward_Slashed(Text(&object, "file")?), root)).or_default();
            }
            Some("value") => values.push(Value_Of(&object, root)?),
            Some("failed") => answer.failed.push(Failed_Of(&object, root)?),
            _ => return Err(Unreadable(&format!("a line of no kind this reader knows: {line}"))),
        }
    }

    for (file, value) in values
    {
        let held = answer.checked.get_mut(&file).ok_or_else(|| return Unreadable(&format!("a value in {file}, which the same answer never names checked")))?;
        held.push(value);
    }
    return Ok(answer);
}

fn Value_Of(object: &serde_json::Value, root: &str) -> Result<(String, DiscardedValue), TypesError>
{
    let file = Relative_To(&Forward_Slashed(Text(object, "file")?), root);
    let is_error = object.get("is_error").and_then(serde_json::Value::as_bool).ok_or_else(|| return Unreadable(&format!("a value with no is_error: {object}")))?;
    let value = DiscardedValue { line: Number(object, "line")?, column: Number(object, "column")?, is_error, type_name: Text(object, "type")?.to_owned() };
    return Ok((file, value));
}

fn Failed_Of(object: &serde_json::Value, root: &str) -> Result<FailedPackage, TypesError>
{
    let files = object.get("files").and_then(serde_json::Value::as_array).ok_or_else(|| return Unreadable(&format!("a failed package with no files: {object}")))?;
    let files = files
        .iter()
        .map(|file| return file.as_str().map(|file| return Relative_To(&Forward_Slashed(file), root)).ok_or_else(|| return Unreadable(&format!("a file that is not text: {file}"))))
        .collect::<Result<Vec<String>, TypesError>>()?;
    let reason = Text(object, "reason")?.split_whitespace().collect::<Vec<&str>>().join(" ");
    return Ok(FailedPackage { package: Text(object, "package")?.to_owned(), files, reason });
}

fn Text<'a>(object: &'a serde_json::Value, field: &str) -> Result<&'a str, TypesError>
{
    return object.get(field).and_then(serde_json::Value::as_str).ok_or_else(|| return Unreadable(&format!("no text {field}: {object}")));
}

fn Number(object: &serde_json::Value, field: &str) -> Result<u32, TypesError>
{
    return object
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .and_then(|number| return u32::try_from(number).ok())
        .ok_or_else(|| return Unreadable(&format!("no number {field}: {object}")));
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

fn Unreadable(what: &str) -> TypesError
{
    return TypesError::New(TypesFailure::Unreadable, &format!("the helper printed something this reader does not know: {what}"));
}

#[cfg(test)]
mod tests;
