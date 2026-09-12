//! Reading `standards.json`'s `limits` and `languages.*.limits` blocks into policy rows.
//!
//! Unlike `crate::naming::reading`, whose `naming` block already existed in this
//! repository's own `standards.json` before its reader was written, no `limits` block
//! existed before this crate: `OD-RULES-011` named the threshold family as its own future
//! instance and left its schema undecided. This item both writes the first `limits` block
//! (with values equal to `nomos-rules`' current hardcoded defaults, so an already-behaving
//! repository regresses nothing) and reads it back, verified directly against this
//! repository's own `standards.json` once written.

use nomos_cap_limits_policy::{PolicyRow, Scope};
use nomos_platform::FileSystem;
use crate::standards_document::{Read_Standards_Document, STANDARDS_JSON};
use std::path::Path;

/// `standards.json` could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LimitsPolicyError
{
    pub reason: String,
}

impl core::fmt::Display for LimitsPolicyError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}

/// Every limits-policy row `root`'s own `standards.json` declares — empty when the file is
/// absent or declares no `limits` block, the same "declaring nothing leaves each rule's
/// own prior default in force" default `crate::naming::Discover_Workspace` already
/// states for the sibling naming capability, since an unconfigured repository is not a
/// repository this capability failed to read.
///
/// # Errors
///
/// [`LimitsPolicyError`] if `standards.json` exists but could not be read for a reason
/// other than absence, is not valid JSON, or declares a `limits` value that is not a
/// non-negative integer.
pub fn Discover_Workspace<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<Vec<PolicyRow>, LimitsPolicyError>
{
    let value = Read_Standards_Document(root, filesystem).map_err(|error| LimitsPolicyError { reason: error.reason })?;

    let mut rows = Vec::new();
    Limits_Rows_Into(&value, &Scope::Repository, &mut rows)?;

    if let Some(languages) = value.get("languages").and_then(serde_json::Value::as_object)
    {
        for (language, declared) in languages
        {
            Limits_Rows_Into(declared, &Scope::Language(language.clone()), &mut rows)?;
        }
    }

    return Ok(Canonical_Order(rows));
}

/// Reads `value`'s own `limits` object, if it has one, into `rows` at `scope`.
fn Limits_Rows_Into(value: &serde_json::Value, scope: &Scope, rows: &mut Vec<PolicyRow>) -> Result<(), LimitsPolicyError>
{
    let Some(limits) = value.get("limits").and_then(serde_json::Value::as_object)
    else
    {
        return Ok(());
    };

    for (key, declared_value) in limits
    {
        let row = Limits_Row(scope, key, declared_value)?;
        rows.push(row);
    }

    return Ok(());
}

/// One `limits.<key>` entry as a [`PolicyRow`] at `scope`, refused if it is not a
/// non-negative integer that fits in a `u32`.
fn Limits_Row(scope: &Scope, key: &str, declared_value: &serde_json::Value) -> Result<PolicyRow, LimitsPolicyError>
{
    let Some(number) = declared_value.as_u64()
    else
    {
        return Err(LimitsPolicyError {
            reason: format!("{STANDARDS_JSON}'s limits.{key} is not a non-negative integer"),
        });
    };

    let value = u32::try_from(number).map_err(|_error| LimitsPolicyError {
        reason: format!("{STANDARDS_JSON}'s limits.{key} is too large: {number}"),
    })?;

    return Ok(PolicyRow { scope: scope.clone(), key: key.to_owned(), value });
}

/// `rows`, in a stable order — neither a JSON object's own representation nor iteration
/// over it promises one, the same reason `crate::naming::reading::Canonical_Order`
/// sorts before encoding.
fn Canonical_Order(mut rows: Vec<PolicyRow>) -> Vec<PolicyRow>
{
    rows.sort_by(|left, right| return (left.scope.Label(), &left.key).cmp(&(right.scope.Label(), &right.key)));

    return rows;
}

#[cfg(test)]
mod tests
{
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
    use super::*;
    use nomos_platform::FileSystemError;
    use nomos_platform_std::StdFileSystem;
    use std::path::PathBuf;

    /// The repository-wide sample threshold these fixtures declare, echoed at both the
    /// encode site and its own assertions so the two ends of a round trip cannot silently
    /// drift apart. Shared by every test below that needs the same value.
    const SAMPLE_REPOSITORY_LIMIT: u32 = 1500;
    /// The per-language sample threshold these fixtures declare.
    const SAMPLE_LANGUAGE_LIMIT: u32 = 1000;
    /// How many rows a fixture declaring one repository-wide and one language row produces.
    const SAMPLE_ROW_COUNT: usize = 2;

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

    #[test]
    fn Test_Discover_Workspace_Should_Read_A_Repository_Wide_And_A_Language_Row()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({
                "limits": { "file-size-hard-lines": SAMPLE_REPOSITORY_LIMIT },
                "languages": { "go": { "limits": { "file-size-hard-lines": SAMPLE_LANGUAGE_LIMIT } } }
            })
            .to_string(),
        };

        let rows = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON");

        assert_eq!(rows.len(), SAMPLE_ROW_COUNT, "{rows:?}");
        assert!(rows.contains(&PolicyRow {
            scope: Scope::Repository,
            key: "file-size-hard-lines".to_owned(),
            value: SAMPLE_REPOSITORY_LIMIT
        }));
        assert!(rows.contains(&PolicyRow {
            scope: Scope::Language("go".to_owned()),
            key: "file-size-hard-lines".to_owned(),
            value: SAMPLE_LANGUAGE_LIMIT
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
    fn Test_Discover_Workspace_Should_Refuse_A_Non_Numeric_Value()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "limits": { "file-size-hard-lines": "many" } }).to_string(),
        };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-numeric value must be refused");

        assert!(error.reason.contains("is not a non-negative integer"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Negative_Value()
    {
        let filesystem = FakeFileSystem {
            text: serde_json::json!({ "limits": { "file-size-hard-lines": -1 } }).to_string(),
        };

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a negative value must be refused");

        assert!(error.reason.contains("is not a non-negative integer"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Return_An_Empty_Policy_For_A_File_Declaring_Neither_Block()
    {
        let filesystem = FakeFileSystem { text: serde_json::json!({ "suppression": {} }).to_string() };

        let rows = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON with no limits block");

        assert!(rows.is_empty(), "{rows:?}");
    }
}
