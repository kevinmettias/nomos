//! Reading the helper's lines against a scripted launcher: the shape measured, each failure sorted,
//! and each other shape refused.

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

/// Names no variable at all.
struct NoEnvironment;

/// Answers from fixed data, so every read is the same.
impl Strategy for NoEnvironment
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Environment for NoEnvironment
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

/// `relative` under [`Root`], spelled as the helper spells it, escaped for a JSON string.
fn At(relative: &str) -> String
{
    let path = PathBuf::from(Root()).join(relative);
    return path.to_string_lossy().replace('\\', "\\\\");
}

fn Exited(code: i32, stdout: &str, stderr: &str) -> ScriptedGo
{
    return ScriptedGo { answer: Ok(ProgramOutput { outcome: ExitOutcome::Exited { code }, stdout: stdout.to_owned(), stderr: stderr.to_owned() }) };
}

fn Typed(go: &ScriptedGo) -> Result<ModuleAnswer, TypesError>
{
    let root = PathBuf::from(Root());
    // The filesystem is never read: running the helper touches only the launcher and the environment.
    let ports = TypesPorts { launcher: go, filesystem: &nomos_platform_std::StdFileSystem, environment: &NoEnvironment };
    return Type_Module(&root.join("helper"), &root, &root.join("app"), &ports);
}

/// The measured shape: checked files, values that lie in them, and a failed package, each path made
/// relative to the root with forward slashes.
#[test]
fn Test_The_Measured_Shape_Should_Be_Read_File_By_File()
{
    let stdout = format!(
        "{{\"file\":\"{main}\",\"kind\":\"checked\"}}\n{{\"file\":\"{quiet}\",\"kind\":\"checked\"}}\n\
         {{\"column\":5,\"file\":\"{main}\",\"is_error\":true,\"kind\":\"value\",\"line\":12,\"type\":\"error\"}}\n\
         {{\"column\":2,\"file\":\"{main}\",\"is_error\":false,\"kind\":\"value\",\"line\":13,\"type\":\"int\"}}\n\
         {{\"files\":[\"{bad}\"],\"kind\":\"failed\",\"package\":\"example.com/app/bad\",\"reason\":\"# example.com/app/bad\\nbad.go:4:2: undefined: x\"}}\n",
        main = At("app/main.go"),
        quiet = At("app/quiet.go"),
        bad = At("app/bad/bad.go"),
    );

    let answer = Typed(&Exited(0, &stdout, "")).expect("the measured shape reads");

    let expected_values = vec![
        DiscardedValue { line: 12, column: 5, is_error: true, type_name: "error".to_owned() },
        DiscardedValue { line: 13, column: 2, is_error: false, type_name: "int".to_owned() },
    ];
    assert_eq!(answer.checked.get("app/main.go"), Some(&expected_values));
    assert_eq!(answer.checked.get("app/quiet.go"), Some(&Vec::new()), "a checked file that discards nothing is an empty answer");
    assert_eq!(
        answer.failed,
        [FailedPackage { package: "example.com/app/bad".to_owned(), files: vec!["app/bad/bad.go".to_owned()], reason: "# example.com/app/bad bad.go:4:2: undefined: x".to_owned() }]
    );
}

/// A line of another kind, a value in a file never named checked, and a value missing a field are
/// each refused rather than read as far as they go.
#[test]
fn Test_Any_Other_Shape_Should_Be_Refused()
{
    let other_kind = "{\"kind\":\"note\",\"file\":\"x\"}\n".to_owned();
    let stray_value = format!("{{\"column\":2,\"file\":\"{}\",\"is_error\":true,\"kind\":\"value\",\"line\":3,\"type\":\"error\"}}\n", At("app/main.go"));
    let short_value = format!("{{\"file\":\"{main}\",\"kind\":\"checked\"}}\n{{\"file\":\"{main}\",\"kind\":\"value\",\"line\":3,\"type\":\"error\"}}\n", main = At("app/main.go"));

    for stdout in [other_kind, stray_value, short_value, "not json\n".to_owned()]
    {
        let refused = Typed(&Exited(0, &stdout, "")).expect_err("a shape this reader does not know is refused");
        assert_eq!(refused.failure, TypesFailure::Unreadable, "{stdout}");
    }
}

/// A helper that could not answer is sorted by why: a toolchain that cannot find its root is
/// unavailable, a module `go list` refuses has failed, a run cut short did not finish, and a `go`
/// that never started is unavailable -- none of them an empty answer.
#[test]
fn Test_Each_Failure_Should_Be_Sorted_By_What_Went_Wrong()
{
    let no_root = Exited(1, "", "go: cannot find GOROOT directory: nowhere");
    let refused = Exited(1, "", "go list: exit status 1: go: go.mod file not found");
    let cut_short = ScriptedGo { answer: Ok(ProgramOutput { outcome: ExitOutcome::TimedOut, stdout: String::new(), stderr: String::new() }) };
    let absent = ScriptedGo { answer: Err("program not found".to_owned()) };

    let failures: Vec<TypesFailure> = [no_root, refused, cut_short, absent].iter().map(|go| return Typed(go).expect_err("no answer").failure).collect();

    assert_eq!(failures, [TypesFailure::GoUnavailable, TypesFailure::Failed, TypesFailure::DidNotFinish, TypesFailure::GoUnavailable]);
}
