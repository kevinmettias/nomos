//! Which revisions the archive holds, in order, and which numbers are missing.

use super::{Path, PathBuf, IngestError};

/// The revision archives, in version order.
///
/// Only the full-suite archives. The topic zips beside them (`nomos_v14_31_ocaml_...`)
/// carry one change apiece and are not revisions of the suite; walking them as if they
/// were would report the whole corpus as disappearing between every pair.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if the directory holds no revision archive, because a
/// directory that matched nothing reads exactly like an archaeology with nothing to say.
pub fn Revisions_In(directory: &Path) -> Result<Vec<(String, PathBuf)>, IngestError>
{
    let mut found = Labelled_Archives(directory)?;
    if found.is_empty()
    {
        return Err(IngestError::Parse(format!(
            "{} holds no revision archive. Refusing to report an archaeology over nothing",
            directory.display()
        )));
    }

    found.sort_by_key(|(label, _)| return Order(label));
    return Ok(found);
}

/// Every archive in the directory whose name gives it a revision label.
pub(super) fn Labelled_Archives(directory: &Path) -> Result<Vec<(String, PathBuf)>, IngestError>
{
    let entries = std::fs::read_dir(directory).map_err(|error| {
        return IngestError::Parse(format!("cannot read {}: {error}", directory.display()));
    })?;

    let mut found: Vec<(String, PathBuf)> = Vec::new();
    for entry in entries.flatten()
    {
        let path = entry.path();
        let name = path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or_default();
        if let Some(label) = Label_Of(name)
        {
            found.push((label, path));
        }
    }

    return Ok(found);
}

/// `nomos-spec-internal-artifacts-v14.36.zip` and `nomos-spec-v15.0.zip`, and nothing else.
pub(super) fn Label_Of(name: &str) -> Option<String>
{
    let stem = name.strip_suffix(".zip")?;
    let label = stem
        .strip_prefix("nomos-spec-internal-artifacts-")
        .or_else(|| return stem.strip_prefix("nomos-spec-"))?;

    return Version(label).map(|_| return label.to_owned());
}

/// A revision label read as the two numbers it is.
///
/// Named rather than a pair of `u32`. The compiler cannot tell a major from a minor, so
/// a call site that took them in the wrong order would sort `v14.36` before `v9.14` and
/// still build — and the ordering these derive is the whole reason the type exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Numbered
{
    pub(super) major: u32,
    pub(super) minor: u32,
}

/// `v14.36` as two numbers, so `v14.9` sorts before `v14.10`.
pub(super) fn Version(label: &str) -> Option<Numbered>
{
    let (major, minor) = label.strip_prefix('v')?.split_once('.')?;

    return Some(Numbered {
        major: major.parse().ok()?,
        minor: minor.parse().ok()?,
    });
}

/// The sort key of a label, with an unreadable one sorting last rather than first.
pub(super) fn Order(label: &str) -> Numbered
{
    return Version(label).unwrap_or(Numbered {
        major: u32::MAX,
        minor: u32::MAX,
    });
}

/// Revision numbers the archive set skips.
///
/// Reported rather than ignored. Two revisions are adjacent among the archives that exist,
/// which is not the same as adjacent in the corpus's own numbering, and a pair spanning a
/// missing revision attributes two revisions' worth of change to one.
#[must_use]
pub fn Gaps(labels: &[String]) -> Vec<String>
{
    let numbered: Vec<Numbered> = labels.iter().filter_map(|label| return Version(label)).collect();
    let mut missing = Vec::new();

    for pair in numbered.windows(2)
    {
        let (Some(before), Some(after)) = (pair.first(), pair.get(1))
        else
        {
            continue;
        };
        let skipped = Between(*before, *after);
        missing.extend(skipped);
    }

    return missing;
}

/// The revisions numbered between two adjacent archives.
///
/// Only within one major version. A major bump is a renumbering rather than a run, so
/// counting from `v14.36` to `v15.0` would report thirty-six revisions nobody ever cut.
pub(super) fn Between(before: Numbered, after: Numbered) -> Vec<String>
{
    if before.major != after.major
    {
        return Vec::new();
    }

    let mut missing = Vec::new();
    let mut minor = before.minor.saturating_add(1);
    while minor < after.minor
    {
        missing.push(format!("v{}.{minor}", before.major));
        minor = minor.saturating_add(1);
    }

    return missing;
}
