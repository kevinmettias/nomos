//! A build set answered against a scripted `MSBuild`: which file is judged under which build, and
//! what one failure withholds.

use super::*;
use crate::EvaluationFailure;
use nomos_contracts::{BuildVariantId, ConfigurationId, DeterminismStrength, Digest128, GenerationId, ReproducibilityScope, SnapshotId, Strategy, TraceEquivalence};
use nomos_platform::{Command, ExitOutcome, ProgramOutput};
use std::ffi::OsString;
use std::path::PathBuf;

/// Two projects: `App` compiles `src/App/A.cs` and `src/Shared/S.cs`, `Tool` compiles
/// `src/Shared/S.cs` alone. `Broken` has no SDK to answer it.
struct ScriptedMsbuild;

/// Answers from fixed data, so the same command always gets the same bytes.
impl Strategy for ScriptedMsbuild
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for ScriptedMsbuild
{
    fn Run(&self, command: &Command) -> Result<ProgramOutput, String>
    {
        let project = command.argv.get(2).map(String::as_str).unwrap_or_default();
        if project.contains("Broken")
        {
            return Err("program not found".to_owned());
        }
        let asks_definitions = command.argv.iter().any(|argument| return argument == "-t:AddImplicitDefineConstants");
        if !asks_definitions
        {
            return Ok(Exited("{\"Properties\": {\"TargetFramework\": \"net8.0\", \"TargetFrameworks\": \"\"}}".to_owned()));
        }

        let compiled: &[&str] = if project.contains("App") { &["src/App/A.cs", "src/Shared/S.cs"] } else { &["src/Shared/S.cs"] };
        let items: Vec<serde_json::Value> = compiled.iter().map(|path| return serde_json::json!({ "FullPath": Full(path) })).collect();
        let answer = serde_json::json!({ "Properties": { "DefineConstants": "DEBUG", "TargetFramework": "net8.0" }, "Items": { "Compile": items } });
        return Ok(Exited(answer.to_string()));
    }
}

/// No variable is set, and the working directory is [`Root`].
struct RootEnvironment;

/// Answers from nothing, so every read is the same.
impl Strategy for RootEnvironment
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Environment for RootEnvironment
{
    fn Variable(&self, _name: &str) -> Option<OsString>
    {
        return None;
    }

    fn Working_Directory(&self) -> Result<PathBuf, nomos_platform::EnvironmentError>
    {
        return Ok(PathBuf::from(Root()));
    }
}

fn Root() -> &'static str
{
    return if cfg!(windows) { "C:\\repo" } else { "/repo" };
}

fn Full(relative: &str) -> String
{
    let separator = if cfg!(windows) { "\\" } else { "/" };
    return format!("{}{separator}{}", Root(), relative.replace('/', separator));
}

fn Exited(stdout: String) -> ProgramOutput
{
    return ProgramOutput { outcome: ExitOutcome::Exited { code: 0 }, stdout, stderr: String::new() };
}

fn Request(project: &str) -> EvaluationRequest
{
    return EvaluationRequest { root: PathBuf::from("."), project: project.to_owned(), configuration: "Debug".to_owned(), target_framework: None };
}

fn File(path: &'static str, text: &'static str) -> FileToJudge<'static>
{
    return FileToJudge { path, subject: SubjectId::From_Digest(nomos_model::Content_Digest(path.as_bytes())), text };
}

fn Context() -> FactContext
{
    return FactContext {
        snapshot: SnapshotId::From_Digest(Digest128::From_Bytes([1; Digest128::BYTE_LENGTH])),
        variant: BuildVariantId::From_Digest(Digest128::From_Bytes([2; Digest128::BYTE_LENGTH])),
        configuration: ConfigurationId::From_Digest(Digest128::From_Bytes([3; Digest128::BYTE_LENGTH])),
        generation: GenerationId::INITIAL,
    };
}

fn Answered(projects: &[&str], files: &[FileToJudge<'static>]) -> BuildSetAnswer
{
    let requests: Vec<EvaluationRequest> = projects.iter().map(|project| return Request(project)).collect();
    return Materialize_Build_Set(&requests, files, Context(), &BuildPorts { launcher: &ScriptedMsbuild, environment: &RootEnvironment });
}

const CONDITIONAL: &str = "#if DEBUG\nclass A {}\n#endif\n";

/// Each file is judged under exactly the builds whose `Compile` items list it: the shared file
/// under both, the app's file under one, and a file neither lists under none.
#[test]
fn Test_Each_File_Should_Be_Judged_Under_Exactly_The_Builds_That_Compile_It()
{
    let files = [File("src/App/A.cs", CONDITIONAL), File("src/Shared/S.cs", CONDITIONAL), File("src/Stray/X.cs", CONDITIONAL)];

    let answer = Answered(&["src/App/App.csproj", "src/Tool/Tool.csproj"], &files);

    let judged: Vec<&str> = answer.facts.iter().map(|fact| return fact.path.as_str()).collect();
    assert_eq!(judged, ["src/App/A.cs", "src/Shared/S.cs", "src/Shared/S.cs"], "{:?}", answer.unjudged);
    assert!(answer.unjudged.is_empty(), "{:?}", answer.unjudged);
    let shared: Vec<_> = answer.facts.iter().filter(|fact| return fact.path == "src/Shared/S.cs").map(|fact| return fact.fact.Key().subject).collect();
    assert_ne!(shared.first(), shared.get(1), "one file under two builds is two subjects");
}

/// One build that cannot be asked withholds every file's facts, not only the files it compiles:
/// the build that did not answer may be the one compiling a branch the others skip.
#[test]
fn Test_A_Build_That_Cannot_Be_Asked_Should_Withhold_Every_Fact()
{
    let files = [File("src/App/A.cs", CONDITIONAL), File("src/Shared/S.cs", CONDITIONAL)];

    let answer = Answered(&["src/App/App.csproj", "src/Broken/Broken.csproj"], &files);

    assert!(answer.facts.is_empty(), "{:?}", answer.facts);
    assert_eq!(answer.unjudged.len(), 1, "{:?}", answer.unjudged);
    let Some(Unjudged::Build { request, error }) = answer.unjudged.first()
    else
    {
        panic!("expected the broken build: {:?}", answer.unjudged);
    };
    assert_eq!(request.project, "src/Broken/Broken.csproj");
    assert_eq!(error.failure, EvaluationFailure::DotnetUnavailable);
}

/// A file refused under a build that compiles it has no fact under any, and every other file is
/// still judged.
#[test]
fn Test_A_File_Refused_Under_One_Build_Should_Have_No_Fact_Under_Any()
{
    let files = [File("src/App/A.cs", CONDITIONAL), File("src/Shared/S.cs", "#if DEBUG\nclass S {}\n")];

    let answer = Answered(&["src/App/App.csproj", "src/Tool/Tool.csproj"], &files);

    let judged: Vec<&str> = answer.facts.iter().map(|fact| return fact.path.as_str()).collect();
    assert_eq!(judged, ["src/App/A.cs"]);
    assert!(matches!(answer.unjudged.as_slice(), [Unjudged::File { path, .. }] if path == "src/Shared/S.cs"), "{:?}", answer.unjudged);
}
