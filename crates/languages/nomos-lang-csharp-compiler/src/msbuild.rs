//! Asking `MSBuild`, through the .NET SDK's own `dotnet msbuild`, which preprocessor symbols a
//! build defines and which files it compiles.
//!
//! Two evaluations, measured against the SDK before this was written. The first reads the
//! project's `TargetFramework` and `TargetFrameworks` with no target run, which is how a
//! multi-targeting project is told apart: it declares the list and leaves the single property
//! empty. The second reads `DefineConstants` and the `Compile` items for one configuration and one
//! framework with the SDK's `AddImplicitDefineConstants` target run, because the framework's own
//! symbols -- `NET8_0`, `NETCOREAPP`, each `_OR_GREATER` -- are added by that target and not by
//! evaluation: without it a `net8.0` Debug build reads as `TRACE;DEBUG` alone. Neither restores,
//! builds or writes anything, and each answered in under a second on the SDKs measured.
//!
//! Asking two properties makes `MSBuild` answer as JSON, `{"Properties": {...}}`; asking one makes
//! it print the bare value. Both calls ask two, so one reader serves both. Asking an item as well
//! adds `"Items": {"Compile": [...]}` beside it, one object per file with its `FullPath`.

use crate::{BuildEvaluation, DefinitionSet, EvaluationError, EvaluationFailure, EvaluationRequest};
use nomos_cap_csharp_semantics::BuildSelection;
use nomos_platform::{Command, Environment, ExitOutcome, ProgramLauncher};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Each evaluation answered in under a second where measured; this is headroom for a cold SDK
/// resolving its workloads, not the expected case.
const TIMEOUT: Duration = Duration::from_secs(120);

/// What the host prints when a .NET runtime is installed and no SDK is: `dotnet` starts, and
/// there is nothing for `msbuild` to run.
const NO_SDK: &str = "No .NET SDKs were found";

/// The definition set `MSBuild` evaluates for `request`'s build, and the files that build compiles.
///
/// # Errors
///
/// [`EvaluationError`] when `dotnet` cannot be started or has no SDK, when `MSBuild` does not
/// finish, or when it refuses the project or the request -- see [`EvaluationFailure`] for which
/// is which. A project targeting several frameworks with none named is refused, never answered
/// for one of them.
pub fn Evaluate_Build<Launcher: ProgramLauncher, Env: Environment>(
    request: &EvaluationRequest,
    launcher: &Launcher,
    environment: &Env,
) -> Result<BuildEvaluation, EvaluationError>
{
    let frameworks = Properties_Of(&Answer(request, None, launcher, environment)?)?;
    let target_framework = Chosen_Framework(request, &frameworks)?;
    let answer = Answer(request, Some(&target_framework), launcher, environment)?;
    let symbols: BTreeSet<String> = Properties_Of(&answer)?
        .get("DefineConstants")
        .map(|constants| return constants.split(';').map(str::trim).filter(|symbol| return !symbol.is_empty()).map(str::to_owned).collect())
        .unwrap_or_default();
    let compiled = Compiled_Of(&answer, &Absolute_Root(request, environment)?)?;

    let selection = BuildSelection { project: request.project.clone(), configuration: request.configuration.clone(), target_framework };
    return Ok(BuildEvaluation { definitions: DefinitionSet::New(selection, symbols), compiled });
}

/// One evaluation's JSON answer: the framework properties when `framework` is `None`, and the
/// definition set and compiled files for that framework otherwise.
fn Answer<Launcher: ProgramLauncher, Env: Environment>(
    request: &EvaluationRequest,
    framework: Option<&str>,
    launcher: &Launcher,
    environment: &Env,
) -> Result<serde_json::Value, EvaluationError>
{
    let command = Msbuild_Command(request, framework, environment);
    let output = launcher
        .Run(&command)
        .map_err(|error| return Failure(EvaluationFailure::DotnetUnavailable, &format!("dotnet could not be started, so this host has no .NET SDK to ask: {error}")))?;

    match output.outcome
    {
        ExitOutcome::Exited { code: 0 } => {}
        ExitOutcome::Exited { code } => return Err(Refusal(code, &output.stdout, &output.stderr)),
        ExitOutcome::TimedOut | ExitOutcome::Stalled { .. } | ExitOutcome::Terminated =>
        {
            return Err(Failure(EvaluationFailure::DidNotFinish, &format!("dotnet msbuild did not finish: {:?}", output.outcome)));
        }
    }

    return Json_Of(&output.stdout);
}

fn Msbuild_Command<Env: Environment>(request: &EvaluationRequest, framework: Option<&str>, environment: &Env) -> Command
{
    let mut argv = vec![Dotnet_Program(environment), "msbuild".to_owned(), request.project.clone(), "-nologo".to_owned(), "-nodeReuse:false".to_owned()];
    argv.push(format!("-p:Configuration={}", request.configuration));
    match framework
    {
        None => argv.extend(["-getProperty:TargetFramework".to_owned(), "-getProperty:TargetFrameworks".to_owned()]),
        Some(framework) => argv.extend([
            "-getProperty:DefineConstants".to_owned(),
            "-getProperty:TargetFramework".to_owned(),
            "-getItem:Compile".to_owned(),
            "-t:AddImplicitDefineConstants".to_owned(),
            format!("-p:TargetFramework={framework}"),
        ]),
    }

    let mut command = Command::From_String_Arguments(argv, TIMEOUT);
    command.working_directory = Some(request.root.clone());
    return command;
}

/// The program `dotnet` is run as: the host the SDK itself names in `DOTNET_HOST_PATH` when this
/// runs under it, and `dotnet` from the path otherwise -- read from the injected environment, so
/// a test's launcher sees exactly the program a real run would start.
fn Dotnet_Program<Env: Environment>(environment: &Env) -> String
{
    return environment.Variable("DOTNET_HOST_PATH").and_then(|value| return value.into_string().ok()).unwrap_or_else(|| return "dotnet".to_owned());
}

/// A finished evaluation that exited non-zero: no SDK when the host says so, a refusal otherwise.
fn Refusal(code: i32, stdout: &str, stderr: &str) -> EvaluationError
{
    let said = format!("{stdout}\n{stderr}");
    if said.contains(NO_SDK)
    {
        return Failure(EvaluationFailure::DotnetUnavailable, &format!("dotnet started and found no SDK: {}", said.trim()));
    }

    let first_error = said.lines().find(|line| return line.contains("error")).unwrap_or(said.trim());
    return Failure(EvaluationFailure::Refused, &format!("dotnet msbuild exited {code}: {}", first_error.trim()));
}

/// `MSBuild`'s JSON answer, read from the first character that opens one -- a first-run banner
/// the SDK may print above it is not part of the answer.
fn Json_Of(stdout: &str) -> Result<serde_json::Value, EvaluationError>
{
    let start = stdout.find('{').ok_or_else(|| return Unreadable(stdout))?;
    return serde_json::from_str(stdout.get(start..).unwrap_or("")).map_err(|_| return Unreadable(stdout));
}

/// The answer's `Properties` object, every value a string.
fn Properties_Of(answer: &serde_json::Value) -> Result<BTreeMap<String, String>, EvaluationError>
{
    let properties = answer.get("Properties").and_then(serde_json::Value::as_object).ok_or_else(|| return Unreadable(&answer.to_string()))?;

    return Ok(properties
        .iter()
        .filter_map(|(name, value)| return value.as_str().map(|value| return (name.clone(), value.to_owned())))
        .collect());
}

/// Every `Compile` item's `FullPath` that sits under `root`, relative to it with forward slashes.
///
/// A missing `Items` object is refused rather than read as a build compiling nothing: this call
/// asked for the items, so an answer without them is not the answer to this question.
fn Compiled_Of(answer: &serde_json::Value, root: &Path) -> Result<BTreeSet<String>, EvaluationError>
{
    let items = answer
        .get("Items")
        .and_then(|items| return items.get("Compile"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| return Unreadable(&answer.to_string()))?;
    let root = Forward_Slashed(&root.to_string_lossy());

    let mut compiled = BTreeSet::new();
    for item in items
    {
        let full = item.get("FullPath").and_then(serde_json::Value::as_str).ok_or_else(|| return Unreadable(&item.to_string()))?;
        if let Some(relative) = Relative_To(&Forward_Slashed(full), &root)
        {
            compiled.insert(relative);
        }
    }

    return Ok(compiled);
}

/// `full` relative to `root`, when it lies beneath it -- compared without regard to ASCII case on
/// Windows, where a root typed as `f:\repos` and one `MSBuild` spells `F:\repos` are one directory.
fn Relative_To(full: &str, root: &str) -> Option<String>
{
    let root = root.trim_end_matches('/');
    let head = full.get(..root.len())?;
    let rest = full.get(root.len()..)?.strip_prefix('/')?;
    let same = if cfg!(windows) { head.eq_ignore_ascii_case(root) } else { head == root };

    return (same && !rest.is_empty()).then(|| return rest.to_owned());
}

/// `path` with backslashes turned forward and every `.` component dropped, so a root given as
/// `.` and resolved against a working directory compares equal to the directory itself.
fn Forward_Slashed(path: &str) -> String
{
    let slashed = path.replace('\\', "/");
    return slashed.split('/').filter(|component| return *component != ".").collect::<Vec<&str>>().join("/");
}

/// The request's root as an absolute path: itself when it is one, and resolved against the
/// injected working directory otherwise -- the directory `MSBuild`'s own `FullPath`s are built
/// from, since it runs there.
fn Absolute_Root<Env: Environment>(request: &EvaluationRequest, environment: &Env) -> Result<PathBuf, EvaluationError>
{
    if request.root.is_absolute()
    {
        return Ok(request.root.clone());
    }

    let working = environment
        .Working_Directory()
        .map_err(|error| return Failure(EvaluationFailure::Refused, &format!("the working directory a relative root is resolved against could not be read: {error}")))?;
    return Ok(working.join(&request.root));
}

/// The one framework to evaluate: the request's, when the project targets it; the project's own,
/// when it declares exactly one; and a refusal otherwise.
fn Chosen_Framework(request: &EvaluationRequest, frameworks: &BTreeMap<String, String>) -> Result<String, EvaluationError>
{
    let single = frameworks.get("TargetFramework").map(|framework| return framework.trim().to_owned()).unwrap_or_default();
    let listed: Vec<String> = frameworks
        .get("TargetFrameworks")
        .map(|list| return list.split(';').map(str::trim).filter(|framework| return !framework.is_empty()).map(str::to_owned).collect())
        .unwrap_or_default();
    let declared: Vec<String> = if single.is_empty() { listed } else { vec![single] };

    return match (&request.target_framework, declared.as_slice())
    {
        (Some(wanted), _) if declared.contains(wanted) => Ok(wanted.clone()),
        (Some(wanted), _) => Err(Failure(EvaluationFailure::Refused, &format!("{} does not target {wanted}; it targets {declared:?}", request.project))),
        (None, [only]) => Ok(only.clone()),
        (None, []) => Err(Failure(EvaluationFailure::Refused, &format!("{} declares no target framework", request.project))),
        (None, several) => Err(Failure(
            EvaluationFailure::Refused,
            &format!("{} targets {several:?}, so a request must name one; each builds with different symbols", request.project),
        )),
    };
}

fn Unreadable(stdout: &str) -> EvaluationError
{
    return Failure(EvaluationFailure::Refused, &format!("dotnet msbuild answered in a shape this reader does not know: {}", stdout.trim()));
}

fn Failure(failure: EvaluationFailure, reason: &str) -> EvaluationError
{
    return EvaluationError { failure, reason: reason.to_owned() };
}

#[cfg(test)]
mod tests;
