//! What the reader makes of a declaration, and what it refuses.

use super::*;
use nomos_platform::{DeterminismStrength, FileSystemError, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform_std::StdFileSystem;
use std::path::PathBuf;

/// Hands back fixed text for any path.
struct FakeFileSystem
{
    text: String,
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for FakeFileSystem
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl FileSystem for FakeFileSystem
{
    fn Read_To_String(&self, _path: &Path) -> Result<String, FileSystemError>
    {
        return Ok(self.text.clone());
    }

    fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
    {
        unimplemented!("this reader never writes")
    }

    fn Exists(&self, _path: &Path) -> bool
    {
        return true;
    }
}

fn Read(declaration: &serde_json::Value) -> Result<Vec<DeclaredBuild>, BuildsDeclarationError>
{
    return Read_Declared_Builds(Path::new("."), &FakeFileSystem { text: declaration.to_string() });
}

fn Build(project: &str, configuration: &str, target_framework: Option<&str>) -> DeclaredBuild
{
    return DeclaredBuild { project: project.to_owned(), configuration: configuration.to_owned(), target_framework: target_framework.map(str::to_owned) };
}

#[test]
fn Test_A_Missing_File_Should_Declare_No_Build()
{
    let builds = Read_Declared_Builds(&PathBuf::from("this/path/does/not/exist"), &StdFileSystem).expect("absence declares nothing");

    assert!(builds.is_empty());
}

/// Each build once, in one order whatever order the file lists them in: a build listed twice is
/// one build, and would otherwise be judged twice and counted as two.
#[test]
fn Test_Declared_Builds_Should_Be_Read_Once_Each_In_A_Fixed_Order()
{
    let declaration = serde_json::json!({ "builds": [
        { "project": "src/App/App.csproj", "configuration": "Release", "target_framework": "net8.0" },
        { "project": "src/App/App.csproj", "configuration": "Debug" },
        { "project": "src/App/App.csproj", "configuration": "Release", "target_framework": "net8.0" },
    ]});

    let builds = Read(&declaration).expect("a well-formed declaration");

    assert_eq!(builds, [Build("src/App/App.csproj", "Debug", None), Build("src/App/App.csproj", "Release", Some("net8.0"))]);
}

/// A misspelled key is a declaration that would not be applied, so it is refused at whichever level
/// it sits -- never skipped, which would judge the build as though the key were absent.
#[test]
fn Test_A_Key_This_Reader_Does_Not_Know_Should_Be_Refused_At_Every_Level()
{
    let top = Read(&serde_json::json!({ "build": [] })).expect_err("a misspelled top-level key");
    let inner = Read(&serde_json::json!({ "builds": [{ "project": "App.csproj", "configuration": "Debug", "target_framwork": "net8.0" }] }))
        .expect_err("a misspelled build key");

    assert!(top.reason.contains("`build`"), "{}", top.reason);
    assert!(inner.reason.contains("`target_framwork`") && inner.reason.contains("build 0"), "{}", inner.reason);
}

#[test]
fn Test_A_Build_Missing_A_Field_Or_Holding_The_Wrong_Kind_Should_Be_Refused()
{
    let no_configuration = Read(&serde_json::json!({ "builds": [{ "project": "App.csproj" }] })).expect_err("no configuration");
    let empty_project = Read(&serde_json::json!({ "builds": [{ "project": " ", "configuration": "Debug" }] })).expect_err("an empty project");
    let numeric_framework = Read(&serde_json::json!({ "builds": [{ "project": "App.csproj", "configuration": "Debug", "target_framework": 8 }] }))
        .expect_err("a framework that is not a string");
    let not_a_list = Read(&serde_json::json!({ "builds": { "project": "App.csproj" } })).expect_err("builds that is not an array");

    assert!(no_configuration.reason.contains("no `configuration`"), "{}", no_configuration.reason);
    assert!(empty_project.reason.contains("`project` is empty"), "{}", empty_project.reason);
    assert!(numeric_framework.reason.contains("`target_framework` is not a string"), "{}", numeric_framework.reason);
    assert!(not_a_list.reason.contains("not an array"), "{}", not_a_list.reason);
}

/// A project is named relative to the root, the way every path this workspace reports is, so an
/// absolute or backslashed one is refused rather than handed to `MSBuild` as something else.
#[test]
fn Test_A_Project_That_Is_Not_A_Relative_Forward_Slashed_Path_Should_Be_Refused()
{
    for project in ["/src/App.csproj", "C:/src/App.csproj", "src\\App.csproj"]
    {
        let refused = Read(&serde_json::json!({ "builds": [{ "project": project, "configuration": "Debug" }] })).expect_err(project);

        assert!(refused.reason.contains("not a path relative to the repository root"), "{project}: {}", refused.reason);
    }
}

/// An object with no `builds` key declares nothing, like an absent file; a file that is not an
/// object at all is not a declaration.
#[test]
fn Test_An_Object_Without_Builds_Should_Declare_None_And_A_Non_Object_Should_Be_Refused()
{
    assert_eq!(Read(&serde_json::json!({})), Ok(Vec::new()));
    assert!(Read(&serde_json::json!([])).is_err());
}
