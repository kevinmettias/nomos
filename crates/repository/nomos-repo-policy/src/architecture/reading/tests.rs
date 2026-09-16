//! What the reader makes of a declaration, and what it refuses.
//!
//! The fixtures declare an architecture this workspace does not have, for the same reason
//! `nomos-cap-architecture`'s own tests do: if these read naturally with components called
//! `Domain` and `Api`, nothing here knows what a zone is.

use super::*;
use nomos_cap_architecture::{Depended, Depending};
use nomos_platform::{DeterminismStrength, FileSystemError, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform_std::StdFileSystem;
use std::path::PathBuf;

/// A value that is not a string. Named because what the fixture is about is the *kind* of
/// value a declaration holds, not the particular number.
const NOT_A_NAME: u8 = 3;

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

fn Declaring(block: serde_json::Value) -> FakeFileSystem
{
    return FakeFileSystem { text: block.to_string() };
}

#[test]
fn Test_Discover_Workspace_Should_Declare_Nothing_For_A_Missing_File()
{
    let payload = Discover_Workspace(&PathBuf::from("this/path/does/not/exist"), &StdFileSystem)
        .expect("a missing declaration file declares nothing rather than failing");

    assert!(!payload.Declares_An_Architecture());
}

/// A file that parses and declares none of the five keys is a repository that has written the
/// file and not filled it in, which reads as declaring nothing rather than as a refusal.
#[test]
fn Test_Discover_Workspace_Should_Declare_Nothing_For_A_File_With_None_Of_The_Keys()
{
    let filesystem = FakeFileSystem { text: serde_json::json!({}).to_string() };

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON with no declared keys");

    assert!(!payload.Declares_An_Architecture());
}

#[test]
fn Test_Discover_Workspace_Should_Read_A_Whole_Declaration()
{
    let filesystem = Declaring(serde_json::json!({
        "components": ["Domain", "Api"],
        "members": { "billing": "Domain", "http": "Api" },
        "permits": { "Api": ["Domain"] },
        "exceptions": { "billing": ["billing-core"] },
        "authorities": { "ledger-store": ["billing"] }
    }));

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

    assert_eq!(payload.components, vec!["Domain".to_owned(), "Api".to_owned()]);
    assert_eq!(payload.Component_Of("billing"), Some("Domain"));
    assert_eq!(payload.Component_Of("http"), Some("Api"));
    assert!(payload.Permits(Depending("Api"), Depended("Domain")));
    assert!(!payload.Permits(Depending("Domain"), Depended("Api")));
    assert!(payload.Excepts(Depending("billing"), Depended("billing-core")));
    assert_eq!(payload.Doors_Into("ledger-store"), Some(["billing".to_owned()].as_slice()));
}

/// The order of a JSON object is not the repository's own statement about anything, so the
/// reader imposes one. Without it the encoded bytes would differ between two reads of the
/// same file, which is the drift `crate::naming::reading::Canonical_Order` sorts against for
/// the same reason.
#[test]
fn Test_Discover_Workspace_Should_Order_Rows_Independently_Of_The_Objects_Own_Order()
{
    let forwards = Declaring(serde_json::json!({
        "components": ["Domain", "Api"],
        "members": { "alpha": "Domain", "zulu": "Api" }
    }));
    let backwards = Declaring(serde_json::json!({
        "components": ["Domain", "Api"],
        "members": { "zulu": "Api", "alpha": "Domain" }
    }));

    let first = Discover_Workspace(Path::new("."), &forwards).expect("well-formed JSON");
    let second = Discover_Workspace(Path::new("."), &backwards).expect("well-formed JSON");

    assert_eq!(first, second);
    assert_eq!(first.membership.first().map(|row| return row.package.as_str()), Some("alpha"));
}

/// `components` keeps the order it was written in, because that order is the repository's own
/// and a reader is shown it -- unlike the four lookups, where an object's order is an artifact.
#[test]
fn Test_Components_Should_Keep_The_Order_They_Were_Declared_In()
{
    let filesystem = Declaring(serde_json::json!({ "components": ["Zulu", "Alpha", "Mike"] }));

    let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

    assert_eq!(payload.components, vec!["Zulu".to_owned(), "Alpha".to_owned(), "Mike".to_owned()]);
}

#[test]
fn Test_Discover_Workspace_Should_Refuse_Invalid_Json()
{
    let filesystem = FakeFileSystem { text: "not json at all".to_owned() };

    let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("invalid JSON must be refused");

    assert!(error.reason.contains("not valid JSON"), "{}", error.reason);
}

#[test]
fn Test_Discover_Workspace_Should_Refuse_A_Components_Value_That_Is_Not_An_Array()
{
    let filesystem = Declaring(serde_json::json!({ "components": "Domain" }));

    let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-array components must be refused");

    assert!(error.reason.contains("components is not an array"), "{}", error.reason);
}

#[test]
fn Test_Discover_Workspace_Should_Refuse_A_Lookup_That_Is_Not_An_Object()
{
    for (key, block) in Non_Object_Lookups()
    {
        let error = Discover_Workspace(Path::new("."), &Declaring(block)).expect_err("a non-object lookup must be refused");
        assert!(error.reason.contains(&format!("{key} is not an object")), "{key}: {}", error.reason);
    }
}

fn Non_Object_Lookups() -> Vec<(&'static str, serde_json::Value)>
{
    return vec![
        ("members", serde_json::json!({ "members": ["billing"] })),
        ("permits", serde_json::json!({ "permits": ["Domain"] })),
        ("exceptions", serde_json::json!({ "exceptions": ["billing"] })),
        ("authorities", serde_json::json!({ "authorities": ["ledger-store"] })),
    ];
}

#[test]
fn Test_Discover_Workspace_Should_Refuse_A_Member_Whose_Component_Is_Not_A_Name()
{
    let filesystem = Declaring(serde_json::json!({ "components": ["Domain"], "members": { "billing": NOT_A_NAME } }));

    let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-string component must be refused");

    assert!(error.reason.contains("is not a name"), "{}", error.reason);
}

#[test]
fn Test_Discover_Workspace_Should_Refuse_A_Permits_Target_List_That_Is_Not_An_Array()
{
    let filesystem = Declaring(serde_json::json!({ "components": ["Domain", "Api"], "permits": { "Api": "Domain" } }));

    let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-array target list must be refused");

    assert!(error.reason.contains("is not an array of names"), "{}", error.reason);
}
