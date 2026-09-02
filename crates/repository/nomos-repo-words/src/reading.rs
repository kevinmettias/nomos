//! Reading `standards.json`'s `words.approved_abbreviations` into a repository's own
//! vocabulary additions.
//!
//! Verified directly against this repository's own `standards.json` before this reader
//! was written: `{"words": {"approved_abbreviations": ["api", "arc", ...]}}`.

use nomos_cap_words_policy::WordsPolicyPayload;
use nomos_platform::{FileSystem, FileSystemError};
use std::path::Path;

const STANDARDS_JSON: &str = "standards.json";

/// `standards.json` could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordsPolicyError
{
    pub reason: String,
}

impl core::fmt::Display for WordsPolicyError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// The vocabulary additions `root`'s own `standards.json` declares — declaring nothing
/// when the file is absent or declares no `words.approved_abbreviations`, since an
/// unconfigured repository is not a repository this capability failed to read.
///
/// # Errors
///
/// [`WordsPolicyError`] if `standards.json` exists but could not be read for a reason
/// other than absence, is not valid JSON, or declares `words.approved_abbreviations` as
/// something other than an array of strings.
pub fn Discover_Workspace<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<WordsPolicyPayload, WordsPolicyError>
{
    let path = root.join(STANDARDS_JSON);
    let text = match filesystem.Read_To_String(&path)
    {
        Ok(text) => text,
        Err(FileSystemError::NotFound { .. }) => return Ok(WordsPolicyPayload::default()),
        Err(error) => {
            return Err(WordsPolicyError {
                reason: format!("{STANDARDS_JSON} could not be read: {error}"),
            });
        }
    };

    let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| WordsPolicyError {
        reason: format!("{STANDARDS_JSON} is not valid JSON: {error}"),
    })?;

    let Some(declared) = value
        .get("words")
        .and_then(|words| return words.get("approved_abbreviations"))
        .and_then(serde_json::Value::as_array)
    else
    {
        return Ok(WordsPolicyPayload::default());
    };

    let mut approved_additions = Vec::new();
    for entry in declared
    {
        let Some(word) = entry.as_str()
        else
        {
            return Err(WordsPolicyError {
                reason: format!("{STANDARDS_JSON}'s words.approved_abbreviations has a non-string entry"),
            });
        };
        approved_additions.push(word.to_owned());
    }

    approved_additions.sort();
    return Ok(WordsPolicyPayload { approved_additions });
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_platform_std::StdFileSystem;
    use std::path::PathBuf;

    #[test]
    fn Test_Discover_Workspace_Should_Read_This_Repositorys_Own_Declared_Additions()
    {
        let payload = Discover_Workspace(&Repository_Root(), &StdFileSystem)
            .expect("this repository's own standards.json is real and well-formed");

        assert!(
            payload.approved_additions.contains(&"kwb".to_owned()),
            "this workspace's own standards.json declares words.approved_abbreviations to include kwb: {payload:?}"
        );
    }

    #[test]
    fn Test_Discover_Workspace_Should_Declare_Nothing_For_A_Missing_File()
    {
        let payload = Discover_Workspace(&PathBuf::from("this/path/does/not/exist"), &StdFileSystem)
            .expect("a missing standards.json declares nothing rather than failing");

        assert_eq!(payload, WordsPolicyPayload::default());
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
            true
        }
    }

    #[test]
    fn Test_Discover_Workspace_Should_Read_A_Declared_List()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "words": { "approved_abbreviations": ["lod", "aabb"] } }).to_string(),
        };

        let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

        assert_eq!(payload.approved_additions, vec!["aabb".to_owned(), "lod".to_owned()], "sorted canonically");
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_Invalid_Json()
    {
        let filesystem = FakeFileSystem { text: "not json at all".to_owned() };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("invalid JSON must be refused");

        assert!(error.reason.contains("not valid JSON"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Non_String_Entry()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "words": { "approved_abbreviations": [5] } }).to_string(),
        };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-string entry must be refused");

        assert!(error.reason.contains("non-string entry"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Return_An_Empty_Policy_For_A_File_Declaring_No_Words_Block()
    {
        let filesystem = FakeFileSystem { text: serde_json::json!({ "suppression": {} }).to_string() };

        let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON with no words block");

        assert_eq!(payload, WordsPolicyPayload::default());
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
}
