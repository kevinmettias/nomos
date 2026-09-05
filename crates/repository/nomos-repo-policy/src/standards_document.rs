//! The shared acquisition step beneath this crate's five `nomos.cap.*.policy` providers.
//!
//! `OD-RULES-019` measured [`crate::naming`], [`crate::limits`], [`crate::scripting`],
//! [`crate::words`] and [`crate::goals`] each independently declaring the same
//! `STANDARDS_JSON` constant, the same `Read_To_String`/`serde_json::from_str` sequence,
//! and a byte-for-byte identical `{ reason: String }` error struct renamed per provider,
//! and decided the physical read/parse step beneath the five is worth sharing while the
//! five capability contracts and providers stay exactly as separate as `OD-RULES-011`
//! built them. `OD-PACKAGE-015` later found the crate boundary `OD-RULES-019` chose for
//! that sharing was not earning its packaging cost — this module is that finding carried
//! out: the same shared step, private to the crate its five siblings now live beside
//! rather than depend on.
//!
//! This module owns only that physical step. [`Read_Standards_Document`] hands back a
//! parsed [`serde_json::Value`], never a typed `naming`/`limits`/`scripting`/`words`/
//! `goals` payload -- semantic extraction of those blocks stays in each provider's own
//! `reading.rs`, unchanged by this module's existence. An absent file is not an error: it
//! reads back as [`serde_json::Value::Null`], which answers `.get(_)` with [`None`] for
//! every key a caller asks it for, the same "declares nothing" meaning each of the five
//! providers already gives a missing file or a missing block.

use nomos_platform::{FileSystem, FileSystemError};
use std::path::Path;

/// The file every `nomos.cap.*.policy` provider reads its configuration from.
pub const STANDARDS_JSON: &str = "standards.json";

/// `standards.json` could not be acquired as a caller expects.
///
/// The one error shape the five providers' own `NamingPolicyError`, `LimitsPolicyError`,
/// `ScriptingPolicyError`, `WordsPolicyError` and `GoalsPolicyError` each convert from
/// rather than re-declare.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StandardsDocumentError
{
    pub reason: String,
}

impl core::fmt::Display for StandardsDocumentError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// `root`'s own `standards.json`, read and parsed -- [`serde_json::Value::Null`] when the
/// file is absent, since an unconfigured repository is not a repository this crate failed
/// to read.
///
/// # Errors
///
/// [`StandardsDocumentError`] if `standards.json` exists but could not be read for a
/// reason other than absence, or is not valid JSON. This function does not look inside the
/// parsed value at all, so it has no opinion on what any block within it declares.
pub fn Read_Standards_Document<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<serde_json::Value, StandardsDocumentError>
{
    let path = root.join(STANDARDS_JSON);
    let text = match filesystem.Read_To_String(&path)
    {
        Ok(text) => text,
        Err(FileSystemError::NotFound { .. }) => return Ok(serde_json::Value::Null),
        Err(error) => {
            return Err(StandardsDocumentError {
                reason: format!("{STANDARDS_JSON} could not be read: {error}"),
            });
        }
    };

    return serde_json::from_str(&text).map_err(|error| StandardsDocumentError {
        reason: format!("{STANDARDS_JSON} is not valid JSON: {error}"),
    });
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_platform_std::StdFileSystem;
    use std::path::PathBuf;

    #[test]
    fn Test_Read_Standards_Document_Should_Read_This_Repositorys_Own_File()
    {
        let value = Read_Standards_Document(&Repository_Root(), &StdFileSystem)
            .expect("this repository's own standards.json is real and well-formed");

        assert_eq!(
            value.get("naming").and_then(|naming| return naming.get("function")).and_then(serde_json::Value::as_str),
            Some("upper-snake"),
            "{value:?}"
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
    fn Test_Read_Standards_Document_Should_Return_Null_For_A_Missing_File()
    {
        let value = Read_Standards_Document(&PathBuf::from("this/path/does/not/exist"), &StdFileSystem)
            .expect("a missing standards.json is not an error");

        assert!(value.is_null(), "{value:?}");
        assert_eq!(value.get("anything"), None, "a Null value must answer every lookup with None");
    }

    /// A [`FileSystem`] that hands back fixed text instead of reading a real path.
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
    fn Test_Read_Standards_Document_Should_Parse_Well_Formed_Json()
    {
        let filesystem = FakeFileSystem { text: serde_json::json!({ "naming": { "function": "upper-snake" } }).to_string() };

        let value = Read_Standards_Document(Path::new("."), &filesystem).expect("well-formed JSON");

        assert_eq!(
            value.get("naming").and_then(|naming| return naming.get("function")).and_then(serde_json::Value::as_str),
            Some("upper-snake")
        );
    }

    #[test]
    fn Test_Read_Standards_Document_Should_Refuse_Invalid_Json()
    {
        let filesystem = FakeFileSystem { text: "not json at all".to_owned() };

        let error = Read_Standards_Document(Path::new("."), &filesystem).expect_err("invalid JSON must be refused");

        assert!(error.reason.contains("not valid JSON"), "{}", error.reason);
    }
}
