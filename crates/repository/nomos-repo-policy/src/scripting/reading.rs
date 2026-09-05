//! Reading `standards.json`'s `scripting.tooling_language` and `scripting.
//! forbidden_extensions` into a scripting policy.
//!
//! Verified directly against this repository's own `standards.json` before this reader
//! was written: `{"scripting": {"tooling_language": "rust", "script_extensions": [],
//! "forbidden_extensions": [".ps1", ".psm1", ".bat", ".cmd", ".sh"]}}`. `script_extensions`
//! is read by no rule this capability powers yet — `Check_Declared_Tooling_Language_For_
//! Scripts`'s own judgment, ported from `check-script-discipline`'s `Is_Forbidden`, needs
//! only the other two fields — so this reader does not carry it; a future rule that does
//! is this capability's own next instance to extend, not a reason to guess at a shape now.

use nomos_cap_scripting_policy::ScriptingPolicyPayload;
use nomos_platform::FileSystem;
use crate::standards_document::{Read_Standards_Document, STANDARDS_JSON};
use std::path::Path;

/// `standards.json` could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptingPolicyError
{
    pub reason: String,
}

impl core::fmt::Display for ScriptingPolicyError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// The scripting policy `root`'s own `standards.json` declares — declaring nothing when
/// the file is absent or declares no `scripting` block, or when `tooling_language` is an
/// empty string: code-standards' own `check-script-discipline` treats an empty declaration
/// as equivalent to none, since an unconfigured repository is not a repository this
/// capability failed to read.
///
/// # Errors
///
/// [`ScriptingPolicyError`] if `standards.json` exists but could not be read for a reason
/// other than absence, is not valid JSON, or declares `tooling_language` or an entry of
/// `forbidden_extensions` that is not a string.
pub fn Discover_Workspace<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<ScriptingPolicyPayload, ScriptingPolicyError>
{
    let value = Read_Standards_Document(root, filesystem).map_err(|error| ScriptingPolicyError { reason: error.reason })?;

    let Some(scripting) = value.get("scripting").and_then(serde_json::Value::as_object)
    else
    {
        return Ok(ScriptingPolicyPayload::default());
    };

    let tooling_language = Tooling_Language_In(scripting)?;
    let forbidden_extensions = Forbidden_Extensions_In(scripting)?;

    return Ok(ScriptingPolicyPayload { tooling_language, forbidden_extensions });
}

fn Tooling_Language_In(scripting: &serde_json::Map<String, serde_json::Value>) -> Result<Option<String>, ScriptingPolicyError>
{
    let Some(declared) = scripting.get("tooling_language") else { return Ok(None) };

    let Some(language) = declared.as_str()
    else
    {
        return Err(ScriptingPolicyError {
            reason: format!("{STANDARDS_JSON}'s scripting.tooling_language is not a string"),
        });
    };

    if language.is_empty()
    {
        return Ok(None);
    }

    return Ok(Some(language.to_owned()));
}

fn Forbidden_Extensions_In(scripting: &serde_json::Map<String, serde_json::Value>) -> Result<Vec<String>, ScriptingPolicyError>
{
    let Some(declared) = scripting.get("forbidden_extensions").and_then(serde_json::Value::as_array) else { return Ok(Vec::new()) };

    let mut extensions = Vec::new();
    for entry in declared
    {
        let Some(extension) = entry.as_str()
        else
        {
            return Err(ScriptingPolicyError {
                reason: format!("{STANDARDS_JSON}'s scripting.forbidden_extensions has a non-string entry"),
            });
        };
        extensions.push(extension.to_owned());
    }

    extensions.sort();
    return Ok(extensions);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_platform::FileSystemError;
    use nomos_platform_std::StdFileSystem;
    use std::path::PathBuf;

    #[test]
    fn Test_Discover_Workspace_Should_Read_This_Repositorys_Own_Declared_Policy()
    {
        let payload = Discover_Workspace(&Repository_Root(), &StdFileSystem)
            .expect("this repository's own standards.json is real and well-formed");

        assert_eq!(payload.tooling_language, Some("rust".to_owned()));
        assert!(
            payload.forbidden_extensions.contains(&".ps1".to_owned()),
            "this workspace's own standards.json declares scripting.forbidden_extensions to include .ps1: {payload:?}"
        );
    }

    fn Repository_Root() -> PathBuf
    {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        return manifest
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .map(PathBuf::from)
            .expect("this crate sits three levels below the workspace root");
    }

    #[test]
    fn Test_Discover_Workspace_Should_Declare_Nothing_For_A_Missing_File()
    {
        let payload = Discover_Workspace(&PathBuf::from("this/path/does/not/exist"), &StdFileSystem)
            .expect("a missing standards.json declares nothing rather than failing");

        assert_eq!(payload, ScriptingPolicyPayload::default());
    }

    /// A [`FileSystem`] that hands back fixed text instead of reading a real path, the
    /// boundary this crate's own module doc names as the one place a caller substitutes a
    /// real filesystem.
    struct FakeFileSystem
    {
        text: String,
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

    #[test]
    fn Test_Discover_Workspace_Should_Read_A_Declared_Language_And_Forbidden_List()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({
                "scripting": { "tooling_language": "go", "forbidden_extensions": [".ps1", ".sh"] }
            })
            .to_string(),
        };

        let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

        assert_eq!(payload.tooling_language, Some("go".to_owned()));
        assert_eq!(payload.forbidden_extensions, vec![".ps1".to_owned(), ".sh".to_owned()]);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Treat_An_Empty_Language_As_Undeclared()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "scripting": { "tooling_language": "" } }).to_string(),
        };

        let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

        assert_eq!(payload.tooling_language, None);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_Invalid_Json()
    {
        let filesystem = FakeFileSystem { text: "not json at all".to_owned() };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("invalid JSON must be refused");

        assert!(error.reason.contains("not valid JSON"), "{}", error.reason);
    }

    /// A value that is not a string, wherever a fixture needs one — its only meaning is
    /// "not a string".
    const NON_STRING_SENTINEL: i64 = 5;

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Non_String_Tooling_Language()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "scripting": { "tooling_language": NON_STRING_SENTINEL } }).to_string(),
        };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-string language must be refused");

        assert!(error.reason.contains("is not a string"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Non_String_Forbidden_Extension()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "scripting": { "forbidden_extensions": [NON_STRING_SENTINEL] } }).to_string(),
        };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-string entry must be refused");

        assert!(error.reason.contains("non-string entry"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Return_An_Empty_Policy_For_A_File_Declaring_No_Scripting_Block()
    {
        let filesystem = FakeFileSystem { text: serde_json::json!({ "suppression": {} }).to_string() };

        let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON with no scripting block");

        assert_eq!(payload, ScriptingPolicyPayload::default());
    }
}
