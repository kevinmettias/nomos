//! What the reader makes of a declaration, and what it refuses.

use super::*;
use nomos_platform::{DeterminismStrength, FileSystemError, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform_std::StdFileSystem;
use std::path::PathBuf;

/// A value that is not a string. Named because what the fixture is about is the *kind* of
/// value a declaration holds, not the particular number.
const NOT_A_LOCATION: u8 = 3;

/// A [`FileSystem`] that hands back fixed text instead of reading a real path, the boundary
/// this module's own doc names as the one place a caller substitutes a real filesystem.
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

fn Declaring_Filesystem(block: serde_json::Value) -> FakeFileSystem
{
    return FakeFileSystem { text: block.to_string() };
}

#[test]
fn Test_Discover_Workspace_Should_Declare_Nothing_For_A_Missing_File()
{
    let payload = Discover_Workspace(&PathBuf::from("this/path/does/not/exist"), &StdFileSystem)
        .expect("a missing declaration file declares nothing rather than failing");

    assert!(payload.locations.is_empty());
}

/// A file that parses and declares no `fixture_locations` key is a repository that has
/// written the file and not filled it in, which reads as declaring nothing rather than as a
/// refusal.
#[test]
fn Test_Discover_Workspace_Should_Declare_Nothing_For_A_File_With_No_Key()
{
    let filesystem = Declaring_Filesystem(serde_json::json!({}));

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON with no declared key");

    assert!(payload.locations.is_empty());
}

#[test]
fn Test_Discover_Workspace_Should_Read_A_Declared_List()
{
    let filesystem = Declaring_Filesystem(serde_json::json!({ "fixture_locations": ["samples", "specimens"] }));

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

    assert_eq!(payload.locations, vec!["samples".to_owned(), "specimens".to_owned()]);
}

/// The order of a JSON array is not the repository's own statement about anything, so the
/// reader imposes one. Without it the encoded bytes would differ between two reads of the
/// same file.
#[test]
fn Test_Discover_Workspace_Should_Order_Locations_Independently_Of_The_Declared_Order()
{
    let forwards = Declaring_Filesystem(serde_json::json!({ "fixture_locations": ["alpha", "zulu"] }));
    let backwards = Declaring_Filesystem(serde_json::json!({ "fixture_locations": ["zulu", "alpha"] }));

    let first = Discover_Workspace(Path::new("."), &forwards).expect("well-formed JSON");
    let second = Discover_Workspace(Path::new("."), &backwards).expect("well-formed JSON");

    assert_eq!(first, second);
}

#[test]
fn Test_Discover_Workspace_Should_Read_This_Repositorys_Own_Declaration()
{
    let payload = Discover_Workspace(&Repository_Root(), &StdFileSystem)
        .expect("this repository's own nomos-test-material.json is real and well-formed");

    assert!(
        payload.locations.is_empty(),
        "this repository declares no fixture locations of its own; the nine toolchain-fixed \
         clauses already cover where its fixtures live: {payload:?}"
    );
}

fn Repository_Root() -> PathBuf
{
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    return manifest
        .ancestors()
        .nth(3)
        .expect("this crate sits three levels below the workspace root")
        .to_path_buf();
}

#[test]
fn Test_Discover_Workspace_Should_Refuse_Invalid_Json()
{
    let filesystem = FakeFileSystem { text: "not json at all".to_owned() };

    let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("invalid JSON must be refused");

    assert!(error.reason.contains("not valid JSON"), "{}", error.reason);
}

#[test]
fn Test_Discover_Workspace_Should_Refuse_A_Fixture_Locations_Value_That_Is_Not_An_Array()
{
    let filesystem = Declaring_Filesystem(serde_json::json!({ "fixture_locations": "samples" }));

    let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-array fixture_locations must be refused");

    assert!(error.reason.contains("fixture_locations is not an array"), "{}", error.reason);
}

#[test]
fn Test_Discover_Workspace_Should_Refuse_A_Non_String_Entry()
{
    let filesystem = Declaring_Filesystem(serde_json::json!({ "fixture_locations": [NOT_A_LOCATION] }));

    let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-string entry must be refused");

    assert!(error.reason.contains("non-string entry"), "{}", error.reason);
}
