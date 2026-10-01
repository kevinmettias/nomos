//! Reading a repository's `nomos-limits.json` into policy rows.
//!
//! # Why its own file, and not `standards.json`
//!
//! This family used to read a `limits` block from `standards.json` and a `limits` block under
//! each of its `languages`. `code-standards` decodes that file into one struct with unknown
//! fields disallowed, and that struct names no `limits` field at either level, so a repository
//! that also runs code-standards could not declare a limit without breaking the other tool --
//! and none did: this repository, xvpe, kwb and code-standards itself each declared none, and
//! every limits rule judged against its compiled default everywhere. `OD-RULES-035` decided a
//! preference is readable only from a place a repository can write it, and moved this family to
//! a file of its own, the convention `OD-HOST-009` set for a declaration this workspace owns
//! outright.
//!
//! # The file's shape
//!
//! The scoping `standards.json` carried, without the `limits` wrapper, because the whole file
//! is this family's: top-level keys are repository-wide limits, each a non-negative integer
//! that fits in a `u32`, and an optional `languages` object maps a language name to an object
//! of that language's own limits. `languages` is the one key that is not a limit. Which keys
//! are limits at all is not decided here -- every key is read, and a key no rule reads is the
//! rules' question.

use nomos_cap_limits_policy::{PolicyRow, Scope};
use nomos_platform::FileSystem;
use crate::standards_document::StandardsDocumentError;
use std::path::Path;

/// The file a repository declares its limits in.
///
/// `nomos-<concern>.json` at the repository root, the convention `nomos-gate.json`,
/// `nomos-architecture.json` and `nomos-test-material.json` already set for a file this
/// workspace owns outright, as against `standards.json`, which it shares.
pub const LIMITS_JSON: &str = "nomos-limits.json";

/// `nomos-limits.json` could not be read as this reader expects.
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

/// Every limits-policy row `root`'s own `nomos-limits.json` declares — empty when the file is
/// absent or declares nothing, the same "declaring nothing leaves each rule's own default in
/// force" answer `crate::naming::Discover_Workspace` gives for the sibling naming capability,
/// since an unconfigured repository is not a repository this capability failed to read.
///
/// # Errors
///
/// [`LimitsPolicyError`] if `nomos-limits.json` exists but could not be read for a reason other
/// than absence, is not valid JSON, or declares a limit that is not a non-negative integer
/// fitting in a `u32`.
pub fn Discover_Workspace<Fs: FileSystem>(root: &Path, filesystem: &Fs) -> Result<Vec<PolicyRow>, LimitsPolicyError>
{
    return crate::scaffolding::Discover_Own_File_Rows(root, filesystem, &crate::scaffolding::ScopedBlock {
        block: LIMITS_JSON,
        repository_scope: Scope::Repository,
        language_scope: |language| Scope::Language(language),
        decode: Limits_Row,
        wrap: |error: StandardsDocumentError| LimitsPolicyError { reason: error.reason },
    })
    .map(Canonical_Order);
}

/// One declared limit as a [`PolicyRow`] at `scope`, refused if it is not a non-negative integer
/// that fits in a `u32`.
fn Limits_Row(scope: &Scope, key: &str, declared_value: &serde_json::Value) -> Result<PolicyRow, LimitsPolicyError>
{
    let Some(number) = declared_value.as_u64()
    else
    {
        return Err(LimitsPolicyError {
            reason: format!("{LIMITS_JSON}'s {key} is not a non-negative integer"),
        });
    };

    let value = u32::try_from(number).map_err(|_error| LimitsPolicyError {
        reason: format!("{LIMITS_JSON}'s {key} is too large: {number}"),
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
    use crate::standards_document::STANDARDS_JSON;
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
            .expect("a missing nomos-limits.json declares nothing rather than failing");

        assert!(rows.is_empty(), "{rows:?}");
    }

    /// A [`FileSystem`] that answers `nomos-limits.json` with `limits` and `standards.json` with
    /// `standards`, and reports every other path absent -- so a test can tell which file the
    /// reader opened, which a fake answering every path alike never could.
    struct FakeFileSystem
    {
        limits: Option<String>,
        standards: Option<String>,
    }

    impl FakeFileSystem
    {
        fn Limits(text: String) -> Self
        {
            return Self { limits: Some(text), standards: None };
        }
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
        fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
        {
            let answer = if path.ends_with(LIMITS_JSON)
            {
                self.limits.clone()
            }
            else if path.ends_with(STANDARDS_JSON)
            {
                self.standards.clone()
            }
            else
            {
                None
            };

            return answer.ok_or_else(|| return FileSystemError::NotFound { path: path.display().to_string() });
        }

        fn Replace_Atomically(&self, _path: &Path, _contents: &str) -> Result<(), FileSystemError>
        {
            unimplemented!("this reader never writes")
        }

        fn Exists(&self, path: &Path) -> bool
        {
            return path.ends_with(LIMITS_JSON) && self.limits.is_some() || path.ends_with(STANDARDS_JSON) && self.standards.is_some();
        }
    }

    #[test]
    fn Test_Discover_Workspace_Should_Read_A_Repository_Wide_And_A_Language_Row()
    {
        let filesystem = FakeFileSystem::Limits(
            serde_json::json!({
                "file-size-hard-lines": SAMPLE_REPOSITORY_LIMIT,
                "languages": { "go": { "file-size-hard-lines": SAMPLE_LANGUAGE_LIMIT } }
            })
            .to_string(),
        );

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

    /// `OD-RULES-035`'s decision, pinned: a `limits` block in `standards.json` is not read, and
    /// the limit this repository declares comes from `nomos-limits.json` alone. Both files
    /// declare the same key with different values, so a reader that still opened the shared
    /// file would report either the wrong value or two rows.
    #[test]
    fn Test_Discover_Workspace_Should_Read_Nomos_Limits_Json_And_Not_Standards_Json()
    {
        let filesystem = FakeFileSystem {
            limits: Some(serde_json::json!({ "file-size-hard-lines": SAMPLE_REPOSITORY_LIMIT }).to_string()),
            standards: Some(serde_json::json!({ "limits": { "file-size-hard-lines": SAMPLE_LANGUAGE_LIMIT } }).to_string()),
        };

        let rows = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON in both files");

        assert_eq!(
            rows,
            vec![PolicyRow { scope: Scope::Repository, key: "file-size-hard-lines".to_owned(), value: SAMPLE_REPOSITORY_LIMIT }]
        );
    }

    /// A repository that still declares its limits only in `standards.json` declares none: the
    /// block is not a second source, and nothing falls back to it.
    #[test]
    fn Test_Discover_Workspace_Should_Declare_Nothing_When_Only_Standards_Json_Has_A_Limits_Block()
    {
        let filesystem = FakeFileSystem {
            limits: None,
            standards: Some(serde_json::json!({ "limits": { "file-size-hard-lines": SAMPLE_REPOSITORY_LIMIT } }).to_string()),
        };

        let rows = Discover_Workspace(Path::new("."), &filesystem).expect("an absent nomos-limits.json declares nothing");

        assert!(rows.is_empty(), "{rows:?}");
    }

    /// `languages` holds rows; it is not itself a limit, so it is neither decoded nor refused
    /// as one.
    #[test]
    fn Test_Discover_Workspace_Should_Not_Read_Languages_As_A_Limit()
    {
        let filesystem = FakeFileSystem::Limits(serde_json::json!({ "languages": { "go": { "parameter-count-max": 5 } } }).to_string());

        let rows = Discover_Workspace(Path::new("."), &filesystem).expect("languages is structure, not a limit");

        assert_eq!(rows, vec![PolicyRow { scope: Scope::Language("go".to_owned()), key: "parameter-count-max".to_owned(), value: 5 }]);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_Invalid_Json()
    {
        let filesystem = FakeFileSystem::Limits("not json at all".to_owned());

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("invalid JSON must be refused");

        assert!(error.reason.starts_with("nomos-limits.json is not valid JSON"), "{}", error.reason);
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Non_Numeric_Value()
    {
        let filesystem = FakeFileSystem::Limits(serde_json::json!({ "file-size-hard-lines": "many" }).to_string());

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a non-numeric value must be refused");

        assert_eq!(error.reason, "nomos-limits.json's file-size-hard-lines is not a non-negative integer");
    }

    #[test]
    fn Test_Discover_Workspace_Should_Refuse_A_Negative_Value()
    {
        let filesystem = FakeFileSystem::Limits(serde_json::json!({ "file-size-hard-lines": -1 }).to_string());

        let error = Discover_Workspace(Path::new("."), &filesystem).expect_err("a negative value must be refused");

        assert_eq!(error.reason, "nomos-limits.json's file-size-hard-lines is not a non-negative integer");
    }

    #[test]
    fn Test_Discover_Workspace_Should_Return_An_Empty_Policy_For_A_File_Declaring_No_Limit()
    {
        let filesystem = FakeFileSystem::Limits(serde_json::json!({}).to_string());

        let rows = Discover_Workspace(Path::new("."), &filesystem).expect("well-formed JSON declaring no limit");

        assert!(rows.is_empty(), "{rows:?}");
    }
}
