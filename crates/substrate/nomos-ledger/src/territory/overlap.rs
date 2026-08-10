//! Whether two territories reserve any of the same ground.

use super::{SubjectId, Subject_Of, Normalize_Path};

/// Every subject two path sets both reach, named once each.
///
/// Deduplicated because two entries reaching one subject is one overlap. Reporting it twice
/// would make a territory that names a directory and a file inside it look like a wider
/// conflict than it is.
pub(super) fn Shared_Subjects(mine: &[String], theirs: &[String]) -> Vec<SubjectId>
{
    let mut shared = Vec::new();

    for left in mine
    {
        for right in theirs
        {
            if !Contains_Or_Equals(left, right)
            {
                continue;
            }

            let narrower = Narrower(left, right);
            let subject = Subject_Of(narrower);
            if !shared.contains(&subject)
            {
                shared.push(subject);
            }
        }
    }

    return shared;
}

/// The more specific of two overlapping paths.
///
/// It names the conflict most usefully: a report saying `crates/a` overlaps is less
/// actionable than one saying `crates/a/src/lib.rs` does.
pub(super) fn Narrower<'a>(left: &'a str, right: &'a str) -> &'a str
{
    if Normalize_Path(left).len() >= Normalize_Path(right).len()
    {
        return left;
    }

    return right;
}

/// Whether one path is the other, or contains it.
///
/// Purely textual, on normalized segments. `a/b` contains `a/b/c`; it does not contain
/// `a/bc`, which is why the comparison appends a separator rather than using a bare
/// `starts_with`.
pub(super) fn Contains_Or_Equals(left: &str, right: &str) -> bool
{
    let left = Normalize_Path(left);
    let right = Normalize_Path(right);

    if left == right
    {
        return true;
    }

    // An empty path is the repository root, which contains everything. It arises from a
    // territory entry of "." or "/", and treating it as a normal name would make the
    // root disjoint from every file in the repository.
    if left.is_empty() || right.is_empty()
    {
        return true;
    }

    return right.starts_with(&format!("{left}/")) || left.starts_with(&format!("{right}/"));
}
