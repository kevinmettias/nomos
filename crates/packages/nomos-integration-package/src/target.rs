//! Whether a materialization intent's target stays inside the repository it names.
//!
//! `OD-PACKAGE-003` declares a target as repository-relative, and a reader that accepted an
//! absolute path or one climbing above the root would be declaring a placement outside the
//! repository the package is integrating -- the one place a generic materializer has no
//! business writing. Judged as text, separator by separator, rather than through
//! `std::path::Path::is_absolute`: that answer differs by host (`/etc` is not absolute on
//! Windows, `C:\` is not on Unix), and a manifest is read on whichever machine happens to
//! hold the checkout, so the refusal has to be the same one everywhere.

/// The two separators a manifest might spell a path with.
const SEPARATORS: [char; 2] = ['/', '\\'];
/// The segment that names the directory it sits in.
const CURRENT_DIRECTORY: &str = ".";
/// The segment that climbs one directory.
const PARENT_DIRECTORY: &str = "..";
/// What follows a drive letter in a Windows drive-qualified path.
const DRIVE_SEPARATOR: char = ':';

/// Whether `target` is anchored somewhere other than the repository root: a leading
/// separator (`/etc`, `\\server\share`) or a drive prefix (`C:\`, and the drive-relative
/// `C:foo`, which Windows resolves against a drive's own current directory).
pub(crate) fn Is_Absolute(target: &str) -> bool
{
    return target.starts_with(SEPARATORS) || Has_Drive_Prefix(target);
}

/// Whether `target` opens with a drive letter and a colon.
fn Has_Drive_Prefix(target: &str) -> bool
{
    let mut characters = target.chars();
    let first = characters.next();
    let second = characters.next();

    return first.is_some_and(|character| return character.is_ascii_alphabetic()) && second == Some(DRIVE_SEPARATOR);
}

/// Whether `target`, walked segment by segment, ever climbs above the root it is relative
/// to. `docs/../AGENTS.md` stays inside; `docs/../../sibling` does not.
pub(crate) fn Escapes_The_Root(target: &str) -> bool
{
    let mut depth: usize = 0;
    for segment in target.split(SEPARATORS)
    {
        let Some(next) = Depth_After(depth, segment)
        else
        {
            return true;
        };

        depth = next;
    }

    return false;
}

/// The depth below the root after one more segment, or `None` once a `..` would climb
/// above it.
fn Depth_After(depth: usize, segment: &str) -> Option<usize>
{
    if segment == PARENT_DIRECTORY
    {
        return depth.checked_sub(1);
    }
    if segment.is_empty() || segment == CURRENT_DIRECTORY
    {
        return Some(depth);
    }

    return Some(depth.saturating_add(1));
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Leading_Separator_Should_Be_Absolute()
    {
        assert!(Is_Absolute("/etc/hosts"));
        assert!(Is_Absolute("\\\\server\\share\\file"));
    }

    #[test]
    fn Test_A_Drive_Prefix_Should_Be_Absolute()
    {
        assert!(Is_Absolute("C:\\Windows"));
        assert!(Is_Absolute("c:/Windows"));
        assert!(Is_Absolute("D:relative-to-a-drive"));
    }

    #[test]
    fn Test_A_Relative_Path_Should_Not_Be_Absolute()
    {
        assert!(!Is_Absolute("AGENTS.md"));
        assert!(!Is_Absolute(".claude/skills"));
        assert!(!Is_Absolute("colon:inside/a/segment"));
    }

    #[test]
    fn Test_Climbing_Above_The_Root_Should_Escape()
    {
        assert!(Escapes_The_Root(".."));
        assert!(Escapes_The_Root("../sibling"));
        assert!(Escapes_The_Root("docs/../../sibling"));
        assert!(Escapes_The_Root("./../sibling"));
    }

    #[test]
    fn Test_Descending_Before_Climbing_Should_Stay_Inside()
    {
        assert!(!Escapes_The_Root("docs/../AGENTS.md"));
        assert!(!Escapes_The_Root("a/b/../c"));
        assert!(!Escapes_The_Root("./AGENTS.md"));
        assert!(!Escapes_The_Root(".github/workflows/gate.yml"));
    }
}
