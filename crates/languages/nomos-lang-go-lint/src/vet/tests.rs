//! Reading `go vet -json` against a scripted launcher: the shape measured, each failure sorted, and
//! each shape refused.

use super::*;
use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform::ProgramOutput;
use std::ffi::OsString;

/// Answers every command with one fixed result, and records nothing.
struct ScriptedGo
{
    answer: Result<ProgramOutput, String>,
}

/// Answers from fixed data, so the same command always gets the same bytes.
impl Strategy for ScriptedGo
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for ScriptedGo
{
    fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
    {
        return self.answer.clone();
    }
}

/// Names `GOROOT` when `root` is given, and nothing else.
struct GoEnvironment
{
    root: Option<&'static str>,
}

/// Answers from fixed data, so every read is the same.
impl Strategy for GoEnvironment
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Environment for GoEnvironment
{
    fn Variable(&self, name: &str) -> Option<OsString>
    {
        return (name == "GOROOT").then_some(self.root).flatten().map(OsString::from);
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

/// `relative` under [`Root`], spelled as `go vet` spells a position on this platform.
fn Position(relative: &str, line: u32, column: u32) -> String
{
    let separator = if cfg!(windows) { "\\" } else { "/" };
    return format!("{}{separator}{}:{line}:{column}", Root(), relative.replace('/', separator));
}

fn Exited(code: i32, stdout: &str, stderr: &str) -> ProgramOutput
{
    return ProgramOutput { outcome: ExitOutcome::Exited { code }, stdout: stdout.to_owned(), stderr: stderr.to_owned() };
}

fn Vetted(answer: Result<ProgramOutput, String>) -> Result<Vec<LintDiagnostic>, VetError>
{
    let root = PathBuf::from(Root());
    return Vet_Module(&root, &root, &ScriptedGo { answer }, &GoEnvironment { root: None });
}

/// The stream as measured: a clean package's `{}`, then a package with two analyzers, one of them
/// carrying a suggested fix this reader has no use for.
fn Measured() -> String
{
    let clean = serde_json::json!({});
    let found = serde_json::json!({ "example.com/probe": {
        "printf": [{ "posn": Position("main.go", 6, 14), "end": Position("main.go", 6, 16), "message": "fmt.Printf format %d has arg \"x\" of wrong type string" }],
        "assign": [{ "posn": Position("main.go", 10, 2), "end": Position("main.go", 10, 2), "message": "self-assignment of x", "suggested_fixes": [] }],
    }});
    return format!("{clean}\n{found}\n");
}

#[test]
fn Test_The_Measured_Stream_Should_Read_As_Sorted_Warnings_Relative_To_The_Root()
{
    let diagnostics = Vetted(Ok(Exited(0, &Measured(), ""))).expect("the measured shape");

    let read: Vec<(String, u32, Option<String>)> = diagnostics.iter().map(|diagnostic| return (diagnostic.file.clone(), diagnostic.line, diagnostic.lint.clone())).collect();
    assert_eq!(read, [("main.go".to_owned(), 6, Some("vet::printf".to_owned())), ("main.go".to_owned(), 10, Some("vet::assign".to_owned()))]);
    assert!(diagnostics.iter().all(|diagnostic| return diagnostic.level == LintLevel::Warning));
}

/// A module whose every package is clean prints only `{}` values: an answer with nothing in it, which
/// is not the same thing as no answer.
#[test]
fn Test_A_Clean_Module_Should_Be_An_Empty_Answer_And_Not_A_Failure()
{
    let diagnostics = Vetted(Ok(Exited(0, "{}\n{}\n", ""))).expect("an answer");

    assert!(diagnostics.is_empty());
}

/// Exit 1 with the compiler's words on stderr is a tool that failed, carrying those words; a `go` that
/// cannot be started, or cannot find its own root, is a toolchain that cannot run; a run that does not
/// end is neither.
#[test]
fn Test_Each_Failure_Should_Be_Sorted_And_None_Should_Read_As_Clean()
{
    let broken = Vetted(Ok(Exited(1, "", "# example.com/broken\nvet.exe: .\\main.go:4:2: undefined: thing\n"))).expect_err("does not type-check");
    let absent = Vetted(Err("program not found".to_owned())).expect_err("no go");
    let trimmed = Vetted(Ok(Exited(2, "", "go: cannot find GOROOT directory: 'go' binary is trimmed and GOROOT is not set"))).expect_err("a go that cannot run");
    let stuck = Vetted(Ok(ProgramOutput { outcome: ExitOutcome::TimedOut, stdout: String::new(), stderr: String::new() })).expect_err("did not finish");

    assert_eq!(broken.failure, VetFailure::Failed);
    assert!(broken.reason.contains("undefined: thing"), "{}", broken.reason);
    assert_eq!(absent.failure, VetFailure::GoUnavailable);
    assert_eq!(trimmed.failure, VetFailure::GoUnavailable);
    assert_eq!(stuck.failure, VetFailure::DidNotFinish);
    assert_eq!(VetFailure::GoUnavailable.Applicability(), nomos_contracts::Applicability::ProviderUnavailable);
    assert_eq!(VetFailure::Failed.Applicability(), nomos_contracts::Applicability::AnalysisFailed);
}

/// A shape this reader was not written for is refused rather than read around -- `go help vet`
/// promises no stability for `-json`, and a guess at a changed shape would be a misread.
#[test]
fn Test_A_Shape_Other_Than_The_Measured_One_Should_Be_Refused()
{
    for stdout in ["not json", "[]", "{\"example.com/x\": []}", "{\"example.com/x\": {\"printf\": [{\"message\": \"no posn\"}]}}", "{\"example.com/x\": {\"printf\": [{\"posn\": \"main.go\", \"message\": \"m\"}]}}"]
    {
        let refused = Vetted(Ok(Exited(0, stdout, ""))).expect_err(stdout);

        assert_eq!(refused.failure, VetFailure::Unreadable, "{stdout}");
    }
}

/// `$GOROOT/bin/go` when the environment names a root, `go` from the path otherwise.
#[test]
fn Test_The_Program_Should_Be_The_Named_Roots_Go_When_One_Is_Named()
{
    let named = Go_Program(&GoEnvironment { root: Some("C:/tools/go") });

    assert_eq!(Go_Program(&GoEnvironment { root: None }), "go");
    assert!(named.starts_with("C:/tools/go") && named.contains("bin"), "{named}");
}
