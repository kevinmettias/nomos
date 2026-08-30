//! Reducing the paths in a change set to names the workspace will accept.

use super::{WorkspaceChangeSet, Change, WorkspaceError};

/// Every change with its path validated and normalized, refusing the whole set on the first
/// bad or repeated path.
///
/// Validated in full before anything is applied, so a set that is refused leaves the
/// workspace exactly as it was — a half-applied checkout is not a state anybody should be
/// able to ask questions about. Normalizing here rather than at each use is also what makes
/// the conflict check see `src/a.rs` and `./src/A.rs` as one path.
pub(super) fn Normalized_Changes(changes: &WorkspaceChangeSet) -> Result<Vec<(String, &Change)>, WorkspaceError>
{
    let mut normalized: Vec<(String, &Change)> = Vec::new();

    for change in changes.Changes()
    {
        let path = Named_Path(change.Path())?;
        if normalized.iter().any(|(seen, _)| return *seen == path)
        {
            return Err(WorkspaceError::Conflicting { path });
        }
        normalized.push((path, change));
    }

    return Ok(normalized);
}

/// Validates and normalizes a submitted path.
pub(super) fn Named_Path(path: &str) -> Result<String, WorkspaceError>
{
    let unified = path.trim().replace('\\', "/");
    let segments = Segments_Of(&unified);

    if let Some(reason) = Unnameable_Path(&unified, &segments)
    {
        return Err(WorkspaceError::Unnamed {
            path: path.to_owned(),
            reason: reason.to_owned(),
        });
    }

    return Ok(segments.join("/").to_lowercase());
}

/// Why a path cannot name a member, if it cannot.
///
/// `..` would let a member address something outside the workspace, and two spellings of
/// one file would be two members.
pub(super) fn Unnameable_Path(unified: &str, segments: &[&str]) -> Option<&'static str>
{
    if Is_Absolute(unified)
    {
        return Some("a member is workspace-relative, and this is absolute");
    }

    if segments.is_empty()
    {
        return Some("it names nothing");
    }

    if segments.contains(&"..")
    {
        return Some("a member cannot reach outside the workspace");
    }

    return None;
}

/// A path's components, with the ones that name nothing dropped.
///
/// `.` and an empty segment both address the directory they sit in, so keeping either would
/// make `src/./a.rs` and `src/a.rs` two members naming one file.
pub(super) fn Segments_Of(unified: &str) -> Vec<&str>
{
    return unified
        .split('/')
        .filter(|segment| return !segment.is_empty() && *segment != ".")
        .collect();
}

/// A path that names a place on one machine rather than a member of a workspace.
///
/// Both spellings are refused: a leading separator and a single-letter drive prefix. A
/// snapshot recording `F:/repos/xvpe/crates/a.rs` is a snapshot that cannot be read anywhere
/// else, and catching it at the door is the difference between a refusal and a corpus of
/// them.
pub(super) fn Is_Absolute(unified: &str) -> bool
{
    return unified.starts_with('/')
        || unified
            .split_once(':')
            .is_some_and(|(prefix, _)| return prefix.len() == 1);
}

/// The same normalization, for lookups that have already been validated elsewhere.
pub(super) fn Normalize_Path(path: &str) -> String
{
    return Named_Path(path).unwrap_or_else(|_| return path.to_owned());
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::ChangeSource;

    #[test]
    fn Test_Normalized_Changes_Should_Reject_The_Same_Member_Named_Twice()
    {
        let changes = WorkspaceChangeSet::From(ChangeSource::GitCheckout)
            .Present("src/a.rs", "one")
            .Present("./SRC/A.RS", "two");

        assert_eq!(
            Normalized_Changes(&changes),
            Err(WorkspaceError::Conflicting {
                path: "src/a.rs".to_owned()
            })
        );
    }

    #[test]
    fn Test_Named_Path_Should_Lowercase_And_Collapse_A_Submitted_Path()
    {
        assert_eq!(Named_Path("./SRC/A.RS").unwrap(), "src/a.rs");
        assert!(Named_Path("../outside.rs").is_err());
    }

    #[test]
    fn Test_Unnameable_Path_Should_Explain_Why_A_Path_Cannot_Name_A_Member()
    {
        assert_eq!(
            Unnameable_Path("/abs.rs", &["abs.rs"]),
            Some("a member is workspace-relative, and this is absolute")
        );
        assert_eq!(Unnameable_Path("src/a.rs", &["src", "a.rs"]), None);
    }

    #[test]
    fn Test_Segments_Of_Should_Drop_Empty_And_Dot_Components()
    {
        assert_eq!(Segments_Of("src//./a.rs"), vec!["src", "a.rs"]);
    }

    #[test]
    fn Test_Is_Absolute_Should_Recognize_A_Leading_Slash_Or_A_Drive_Letter()
    {
        assert!(Is_Absolute("/a.rs"));
        assert!(Is_Absolute("f:/a.rs"));
        assert!(!Is_Absolute("src/a.rs"));
    }

    #[test]
    fn Test_Normalize_Path_Should_Fall_Back_To_The_Original_When_Unnameable()
    {
        assert_eq!(Normalize_Path("SRC/A.RS"), "src/a.rs");
        assert_eq!(Normalize_Path("../outside.rs"), "../outside.rs");
    }
}
