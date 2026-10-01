//! The evaluation against a scripted launcher: the commands it builds, each answer it reads, and
//! each failure it sorts.

use crate::{BuildEvaluation, Evaluate_Build, EvaluationError, EvaluationFailure, EvaluationRequest};
use nomos_contracts::{Applicability, DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform::{Command, Environment, ExitOutcome, ProgramLauncher, ProgramOutput};
use std::ffi::OsString;
use std::path::PathBuf;

/// Answers from fixed data: the framework question with `frameworks`, the definition question
/// with `definitions` -- told apart by whether the command runs the implicit-constants target.
struct ScriptedLauncher
{
    frameworks: Result<ProgramOutput, String>,
    definitions: Result<ProgramOutput, String>,
}

/// Answers from fixed data, so the same command always gets the same bytes.
impl Strategy for ScriptedLauncher
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for ScriptedLauncher
{
    fn Run(&self, command: &Command) -> Result<ProgramOutput, String>
    {
        let asks_definitions = command.argv.iter().any(|argument| return argument == "-t:AddImplicitDefineConstants");
        return if asks_definitions { self.definitions.clone() } else { self.frameworks.clone() };
    }
}

/// No variable is set, and the working directory is [`Work`].
struct EmptyEnvironment;

/// Answers from nothing, so every read is the same.
impl Strategy for EmptyEnvironment
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Environment for EmptyEnvironment
{
    fn Variable(&self, _name: &str) -> Option<OsString>
    {
        return None;
    }

    fn Working_Directory(&self) -> Result<PathBuf, nomos_platform::EnvironmentError>
    {
        return Ok(PathBuf::from(Work()));
    }
}

/// An absolute directory on this platform, standing in for the repository root.
fn Work() -> &'static str
{
    return if cfg!(windows) { "C:\\work" } else { "/work" };
}

/// `relative` under [`Work`], spelled the way `MSBuild` spells a `FullPath` on this platform.
fn Full(relative: &str) -> String
{
    let separator = if cfg!(windows) { "\\" } else { "/" };
    return format!("{}{separator}{}", Work(), relative.replace('/', separator));
}

fn Exited(code: i32, stdout: &str) -> ProgramOutput
{
    return ProgramOutput { outcome: ExitOutcome::Exited { code }, stdout: stdout.to_owned(), stderr: String::new() };
}

/// A request rooted at `.`, which the evaluation resolves against [`Work`] -- so every test here
/// also exercises a relative root.
fn Request(framework: Option<&str>) -> EvaluationRequest
{
    return EvaluationRequest {
        root: PathBuf::from("."),
        project: "src/App/App.csproj".to_owned(),
        configuration: "Debug".to_owned(),
        target_framework: framework.map(str::to_owned),
    };
}

const SINGLE: &str = "{\"Properties\": {\"TargetFramework\": \"net8.0\", \"TargetFrameworks\": \"\"}}";
const MULTI: &str = "{\"Properties\": {\"TargetFramework\": \"\", \"TargetFrameworks\": \"net8.0;netstandard2.0\"}}";

/// The definition answer: three symbols with a blank and a duplicate among them, and `compiled`'s
/// files as `Compile` items -- each an absolute `FullPath`, as `MSBuild` reports them.
fn Defined(compiled: &[String]) -> String
{
    let items: Vec<serde_json::Value> = compiled.iter().map(|full| return serde_json::json!({ "Identity": "x.cs", "FullPath": full })).collect();
    let answer = serde_json::json!({
        "Properties": { "DefineConstants": "TRACE;DEBUG;NET8_0; ;NET8_0", "TargetFramework": "net8.0" },
        "Items": { "Compile": items },
    });
    return answer.to_string();
}

fn Defined_One() -> String
{
    return Defined(&[Full("src/App/A.cs")]);
}

fn Evaluated(frameworks: Result<ProgramOutput, String>, definitions: Result<ProgramOutput, String>, framework: Option<&str>) -> Result<BuildEvaluation, EvaluationError>
{
    return Evaluate_Build(&Request(framework), &ScriptedLauncher { frameworks, definitions }, &EmptyEnvironment);
}

#[test]
fn Test_A_Single_Target_Project_Should_Evaluate_To_Its_Symbols_Deduplicated()
{
    let set = Evaluated(Ok(Exited(0, SINGLE)), Ok(Exited(0, &Defined_One())), None).expect("a single-target project").definitions;

    assert_eq!(set.selection.target_framework, "net8.0");
    assert_eq!(set.symbols.iter().map(String::as_str).collect::<Vec<_>>(), ["DEBUG", "NET8_0", "TRACE"]);
}

/// The files a build compiles are `MSBuild`'s `Compile` items, relative to the root: a file under a
/// nested project's directory is listed when the outer project's glob takes it, and one the build
/// takes from outside the root is not listed, since nothing under the root names it.
#[test]
fn Test_The_Compiled_Files_Should_Be_Msbuilds_Items_Relative_To_The_Root()
{
    let outside = if cfg!(windows) { "C:\\elsewhere\\Shared.cs".to_owned() } else { "/elsewhere/Shared.cs".to_owned() };
    let answer = Defined(&[Full("src/App/A.cs"), Full("src/App/Nested/N.cs"), outside]);

    let evaluation = Evaluated(Ok(Exited(0, SINGLE)), Ok(Exited(0, &answer)), None).expect("a single-target project");

    assert_eq!(evaluation.compiled.iter().map(String::as_str).collect::<Vec<_>>(), ["src/App/A.cs", "src/App/Nested/N.cs"]);
    assert!(evaluation.Compiles("src/App/Nested/N.cs"));
    assert!(!evaluation.Compiles("src/Other/B.cs"));
}

/// The definition question asked for the items, so an answer without them is not an answer to it --
/// never read as a build that compiles nothing.
#[test]
fn Test_An_Answer_Without_Its_Items_Should_Be_Refused_Rather_Than_Read_As_Compiling_Nothing()
{
    let bare = "{\"Properties\": {\"DefineConstants\": \"DEBUG\", \"TargetFramework\": \"net8.0\"}}";

    let refused = Evaluated(Ok(Exited(0, SINGLE)), Ok(Exited(0, bare)), None).expect_err("no Items object");

    assert_eq!(refused.failure, EvaluationFailure::Refused);
}

/// A banner the SDK prints above its answer on a first run is not part of the answer.
#[test]
fn Test_A_Banner_Above_The_Answer_Should_Be_Read_Past()
{
    let banner = format!("Welcome to .NET!\n---------------\n{SINGLE}");

    assert!(Evaluated(Ok(Exited(0, &banner)), Ok(Exited(0, &Defined_One())), None).is_ok());
}

/// Several targets and none named has no single answer, so it is refused rather than answered
/// for whichever is listed first; naming one answers for that one.
#[test]
fn Test_A_Multi_Target_Project_Should_Be_Refused_Unless_The_Request_Names_One()
{
    let defined = Defined_One();
    let refused = Evaluated(Ok(Exited(0, MULTI)), Ok(Exited(0, &defined)), None).expect_err("two frameworks, none named");
    let chosen = Evaluated(Ok(Exited(0, MULTI)), Ok(Exited(0, &defined)), Some("net8.0")).expect("one named");
    let absent = Evaluated(Ok(Exited(0, SINGLE)), Ok(Exited(0, &defined)), Some("net48")).expect_err("a framework the project does not target");

    assert_eq!(refused.failure, EvaluationFailure::Refused);
    assert!(refused.reason.contains("netstandard2.0"), "{}", refused.reason);
    assert_eq!(chosen.definitions.selection.target_framework, "net8.0");
    assert!(absent.reason.contains("does not target net48"), "{}", absent.reason);
}

#[test]
fn Test_Each_Failure_Should_Be_Sorted_By_What_A_Person_Can_Do_About_It()
{
    let defined = Defined_One();
    let not_started = Evaluated(Err("program not found".to_owned()), Ok(Exited(0, &defined)), None).expect_err("no dotnet");
    let no_sdk = Evaluated(Ok(Exited(145, "No .NET SDKs were found.")), Ok(Exited(0, &defined)), None).expect_err("a runtime with no SDK");
    let timed_out = Evaluated(Ok(ProgramOutput { outcome: ExitOutcome::TimedOut, stdout: String::new(), stderr: String::new() }), Ok(Exited(0, &defined)), None)
        .expect_err("a stuck MSBuild");
    let refused = Evaluated(Ok(Exited(1, "App.csproj : error MSB4025: The project file could not be loaded.")), Ok(Exited(0, &defined)), None)
        .expect_err("a malformed project");
    let unreadable = Evaluated(Ok(Exited(0, "TRACE;DEBUG")), Ok(Exited(0, &defined)), None).expect_err("a bare value where JSON was asked for");

    assert_eq!(not_started.failure.Applicability(), Applicability::DependencyUnavailable);
    assert_eq!(no_sdk.failure.Applicability(), Applicability::DependencyUnavailable);
    assert_eq!(timed_out.failure.Applicability(), Applicability::ProviderUnavailable);
    assert_eq!(refused.failure.Applicability(), Applicability::AnalysisFailed);
    assert!(refused.reason.contains("MSB4025"), "{}", refused.reason);
    assert_eq!(unreadable.failure, EvaluationFailure::Refused);
}
