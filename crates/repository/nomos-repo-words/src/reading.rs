//! Reading `standards.json`'s `words.approved_abbreviations`, `words.vague` and
//! `words.vague_exempt` into a repository's own vocabulary additions and exemptions.
//!
//! Verified directly against this repository's own `standards.json` before this reader
//! was written: `{"words": {"approved_abbreviations": ["api", "arc", ...]}}`. `vague`
//! and `vague_exempt` are read the identical way, mirroring code-standards' own
//! `Config.Vague`/`Config.Vague_Exempt` JSON tags.

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

/// The vocabulary additions and exemptions `root`'s own `standards.json` declares —
/// declaring nothing when the file is absent or declares no `words` block at all, since an
/// unconfigured repository is not a repository this capability failed to read.
///
/// # Errors
///
/// [`WordsPolicyError`] if `standards.json` exists but could not be read for a reason
/// other than absence, is not valid JSON, or declares `words.approved_abbreviations`,
/// `words.vague` or `words.vague_exempt` as something other than an array of strings.
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

    let words = value.get("words");
    let approved_additions = Read_Word_Array(words, "approved_abbreviations")?;
    let vague_additions = Read_Word_Array(words, "vague")?;
    let vague_exempt = Read_Word_Array(words, "vague_exempt")?;

    return Ok(WordsPolicyPayload { approved_additions, vague_additions, vague_exempt });
}

/// `words.<key>` as a sorted list of strings, or an empty list when `words` or `words.<key>`
/// is absent — the same "absence declares nothing" reading each of the three keys shares.
fn Read_Word_Array(words: Option<&serde_json::Value>, key: &str) -> Result<Vec<String>, WordsPolicyError>
{
    let Some(declared) = words.and_then(|words| return words.get(key)).and_then(serde_json::Value::as_array)
    else
    {
        return Ok(Vec::new());
    };

    let mut collected = Vec::new();
    for entry in declared
    {
        let Some(word) = entry.as_str()
        else
        {
            return Err(WordsPolicyError {
                reason: format!("{STANDARDS_JSON}'s words.{key} has a non-string entry"),
            });
        };
        collected.push(word.to_owned());
    }

    collected.sort();
    return Ok(collected);
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
    fn Test_Discover_Workspace_Should_Read_A_Declared_Vague_Addition_And_Exemption()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "words": { "vague": ["registry"], "vague_exempt": ["info"] } }).to_string(),
        };

        let payload = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

        assert_eq!(payload.vague_additions, vec!["registry".to_owned()]);
        assert_eq!(payload.vague_exempt, vec!["info".to_owned()]);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Non_String_Vague_Entry()
    {
        let filesystem = FakeFileSystem { text: serde_json::json!({ "words": { "vague": [5] } }).to_string() };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-string entry must be refused");

        assert!(error.reason.contains("words.vague"), "{}", error.reason);
        assert!(error.reason.contains("non-string entry"), "{}", error.reason);
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
