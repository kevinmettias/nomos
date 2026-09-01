//! Reading `standards.json`'s `naming` and `languages.*.naming` blocks into policy rows.
//!
//! Verified directly against this repository's own `standards.json` before this reader was
//! written: `{"naming": {"function": "upper-snake", ...}, "languages": {"rust": {"naming":
//! {...}}}}`, the same schema-validated shape its own `data_contracts` entry already
//! requires as an `object` field.

use nomos_cap_naming_policy::{Case, PolicyRow, Scope};
use nomos_platform::{FileSystem, FileSystemError};
use std::path::Path;

const STANDARDS_JSON: &str = "standards.json";

/// `standards.json` could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamingPolicyError
{
    pub reason: String,
}

impl core::fmt::Display for NamingPolicyError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// Every naming-policy row `root`'s own `standards.json` declares — empty when the file is
/// absent, the same "declaring nothing leaves each language's own convention in force"
/// default `check-naming`'s own `spec.go` states, since an unconfigured repository is not
/// a repository this capability failed to read.
///
/// # Errors
///
/// [`NamingPolicyError`] if `standards.json` exists but could not be read for a reason
/// other than absence, is not valid JSON, or declares a `naming` value that is not a
/// string or names a case outside [`Case`]'s closed set — a casing scheme this reader
/// cannot read is one it would silently ignore, and this reader refuses instead.
pub fn Discover_Workspace<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<Vec<PolicyRow>, NamingPolicyError>
{
    let path = root.join(STANDARDS_JSON);
    let text = match filesystem.Read_To_String(&path)
    {
        Ok(text) => text,
        Err(FileSystemError::NotFound { .. }) => return Ok(Vec::new()),
        Err(error) => {
            return Err(NamingPolicyError {
                reason: format!("{STANDARDS_JSON} could not be read: {error}"),
            });
        }
    };

    let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| NamingPolicyError {
        reason: format!("{STANDARDS_JSON} is not valid JSON: {error}"),
    })?;

    let mut rows = Vec::new();
    Naming_Rows_Into(&value, &Scope::Repository, &mut rows)?;

    if let Some(languages) = value.get("languages").and_then(serde_json::Value::as_object)
    {
        for (language, declared) in languages
        {
            Naming_Rows_Into(declared, &Scope::Language(language.clone()), &mut rows)?;
        }
    }

    return Ok(Canonical_Order(rows));
}

/// Reads `value`'s own `naming` object, if it has one, into `rows` at `scope`.
fn Naming_Rows_Into(value: &serde_json::Value, scope: &Scope, rows: &mut Vec<PolicyRow>) -> Result<(), NamingPolicyError>
{
    let Some(naming) = value.get("naming").and_then(serde_json::Value::as_object)
    else
    {
        return Ok(());
    };

    for (symbol, declared_case) in naming
    {
        let Some(label) = declared_case.as_str()
        else
        {
            return Err(NamingPolicyError {
                reason: format!("{STANDARDS_JSON}'s naming.{symbol} is not a string"),
            });
        };

        let Some(case) = Case::From_Label(label)
        else
        {
            return Err(NamingPolicyError {
                reason: format!("{STANDARDS_JSON}'s naming.{symbol} names an unrecognized case {label:?}"),
            });
        };

        rows.push(PolicyRow { scope: scope.clone(), symbol: symbol.clone(), case });
    }

    return Ok(());
}

/// `rows`, in a stable order — neither a JSON object's own representation nor iteration
/// over it promises one, the same reason `nomos_lang_rust_deny::deny_error::Canonical_
/// Order` sorts before encoding.
fn Canonical_Order(mut rows: Vec<PolicyRow>) -> Vec<PolicyRow>
{
    rows.sort_by(|left, right| return (left.scope.Label(), &left.symbol).cmp(&(right.scope.Label(), &right.symbol)));

    return rows;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_platform_std::StdFileSystem;
    use std::path::PathBuf;

    #[test]
    fn Test_Discover_Workspace_Should_Read_This_Repositorys_Own_Declared_Convention()
    {
        let rows = Discover_Workspace(&Repository_Root(), &StdFileSystem)
            .expect("this repository's own standards.json is real and well-formed");

        assert!(
            rows.iter().any(|row| return row.scope == Scope::Repository && row.symbol == "function" && row.case == Case::UpperSnake),
            "this workspace's own standards.json declares naming.function = \"upper-snake\": {rows:?}"
        );
        assert!(
            rows.iter().any(|row| {
                return row.scope == Scope::Language("rust".to_owned()) && row.symbol == "function" && row.case == Case::UpperSnake;
            }),
            "this workspace's own standards.json declares languages.rust.naming.function = \"upper-snake\": {rows:?}"
        );
    }

    #[test]
    fn Test_Discover_Workspace_Should_Declare_Nothing_For_A_Missing_File()
    {
        let rows = Discover_Workspace(&PathBuf::from("this/path/does/not/exist"), &StdFileSystem)
            .expect("a missing standards.json declares nothing rather than failing");

        assert!(rows.is_empty(), "{rows:?}");
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
    fn Test_Discover_Workspace_Should_Read_A_Repository_Wide_And_A_Language_Row()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({
                "naming": { "function": "upper-snake" },
                "languages": { "go": { "naming": { "function": "screaming-snake" } } }
            })
            .to_string(),
        };

        let rows = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

        assert_eq!(rows.len(), 2, "{rows:?}");
        assert!(rows.contains(&PolicyRow { scope: Scope::Repository, symbol: "function".to_owned(), case: Case::UpperSnake }));
        assert!(rows.contains(&PolicyRow {
            scope: Scope::Language("go".to_owned()),
            symbol: "function".to_owned(),
            case: Case::ScreamingSnake
        }));
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_Invalid_Json()
    {
        let filesystem = FakeFileSystem { text: "not json at all".to_owned() };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("invalid JSON must be refused");

        assert!(error.reason.contains("not valid JSON"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_An_Unrecognized_Case()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "naming": { "function": "PascalCase" } }).to_string(),
        };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("an unrecognized case must be refused");

        assert!(error.reason.contains("unrecognized case"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Non_String_Case_Value()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "naming": { "function": 5 } }).to_string(),
        };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-string case value must be refused");

        assert!(error.reason.contains("is not a string"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Return_An_Empty_Policy_For_A_File_Declaring_Neither_Block()
    {
        let filesystem = FakeFileSystem { text: serde_json::json!({ "suppression": {} }).to_string() };

        let rows = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON with no naming block");

        assert!(rows.is_empty(), "{rows:?}");
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
