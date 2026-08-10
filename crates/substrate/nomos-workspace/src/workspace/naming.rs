//! Reducing the paths in a change set to names the workspace will accept.

use super::{WorkspaceChangeSet, Change, WorkspaceError};

/// Every change with its path validated and normalized, refusing the whole set on the first
/// bad or repeated path.
///
/// Validated in full before anything is applied, so a set that is refused leaves the
/// workspace exactly as it was — a half-applied checkout is not a state anybody should be
/// able to ask questions about. Normalizing here rather than at each use is also what makes
/// the conflict check see `src/a.rs` and `./src/A.rs` as one path.
pub(super) fn Normalized(changes: &WorkspaceChangeSet) -> Result<Vec<(String, &Change)>, WorkspaceError>
{
    let mut normalized: Vec<(String, &Change)> = Vec::new();

    for change in changes.Changes()
    {
        let path = Named(change.Path())?;
        if normalized.iter().any(|(seen, _)| return *seen == path)
        {
            return Err(WorkspaceError::Conflicting { path });
        }
        normalized.push((path, change));
    }

    return Ok(normalized);
}

/// Validates and normalizes a submitted path.
pub(super) fn Named(path: &str) -> Result<String, WorkspaceError>
{
    let unified = path.trim().replace('\\', "/");
    let segments = Segments_Of(&unified);

    if let Some(reason) = Unnameable(&unified, &segments)
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
pub(super) fn Unnameable(unified: &str, segments: &[&str]) -> Option<&'static str>
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
pub(super) fn Normalize(path: &str) -> String
{
    return Named(path).unwrap_or_else(|_| return path.to_owned());
}
