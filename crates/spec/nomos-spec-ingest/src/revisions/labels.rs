//! Which revisions the archive holds, in order, and which numbers are missing.

use super::{Path, PathBuf, IngestError};

/// A window over two neighbours, which is what "adjacent" means to a sequence.
const ADJACENT_PAIR: usize = 2;

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

    found.sort_by_key(|(label, _)| return Order_Of(label));
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

    return Version_Of(label).map(|_| return label.to_owned());
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
pub(super) fn Version_Of(label: &str) -> Option<Numbered>
{
    let (major, minor) = label.strip_prefix('v')?.split_once('.')?;

    return Some(Numbered {
        major: major.parse().ok()?,
        minor: minor.parse().ok()?,
    });
}

/// The sort key of a label, with an unreadable one sorting last rather than first.
pub(super) fn Order_Of(label: &str) -> Numbered
{
    return Version_Of(label).unwrap_or(Numbered {
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
pub fn Label_Gaps(labels: &[String]) -> Vec<String>
{
    let numbered: Vec<Numbered> = labels.iter().filter_map(|label| return Version_Of(label)).collect();
    let mut missing = Vec::new();

    for pair in numbered.windows(ADJACENT_PAIR)
    {
        let (Some(before), Some(after)) = (pair.first(), pair.get(1))
        else
        {
            continue;
        };
        let skipped = Missing_Between(*before, *after);
        missing.extend(skipped);
    }

    return missing;
}

/// The revisions numbered between two adjacent archives.
///
/// Only within one major version. A major bump is a renumbering rather than a run, so
/// counting from `v14.36` to `v15.0` would report thirty-six revisions nobody ever cut.
pub(super) fn Missing_Between(before: Numbered, after: Numbered) -> Vec<String>
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

#[cfg(test)]
mod tests
{
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn Test_Revisions_In_Should_Sort_By_Numeric_Label_Order()
    {
        let directory = Temporary_Directory_With("sort-order", &["nomos-spec-v14.36.zip", "nomos-spec-v14.9.zip"]);

        let revisions = Revisions_In(&directory).expect("finds the archives");
        let labels: Vec<&str> = revisions.iter().map(|(label, _)| return label.as_str()).collect();

        assert_eq!(labels, vec!["v14.9", "v14.36"]);
    }

    #[test]
    fn Test_Revisions_In_Should_Refuse_A_Directory_With_No_Revision_Archive()
    {
        let directory = Temporary_Directory_With("empty", &["readme.txt"]);

        let refusal = Revisions_In(&directory).expect_err("must refuse");

        assert!(matches!(refusal, IngestError::Parse(_)), "{refusal}");
    }

    #[test]
    fn Test_Labelled_Archives_Should_Collect_Every_Archive_The_Directory_Names()
    {
        let directory = Temporary_Directory_With(
            "labelled",
            &["nomos-spec-v14.36.zip", "nomos-spec-internal-artifacts-v14.37.zip", "readme.txt"],
        );

        let found = Labelled_Archives(&directory).expect("reads the directory");
        let labels: BTreeSet<&str> = found.iter().map(|(label, _)| return label.as_str()).collect();

        assert_eq!(labels.len(), 2);
        assert!(labels.contains("v14.36"));
        assert!(labels.contains("v14.37"));
    }

    #[test]
    fn Test_Label_Of_Should_Recognize_A_Revision_Archive_By_Name()
    {
        assert_eq!(Label_Of("nomos-spec-v15.0.zip"), Some("v15.0".to_owned()));
        assert_eq!(Label_Of("nomos_v14_31_ocaml_semantic_kernel.zip"), None);
    }

    #[test]
    fn Test_Version_Of_Should_Parse_Major_And_Minor_As_Numbers()
    {
        assert_eq!(Version_Of("v14.36"), Some(Numbered { major: 14, minor: 36 }));
        assert_eq!(Version_Of("not-a-version"), None);
    }

    #[test]
    fn Test_Order_Of_Should_Sort_An_Unreadable_Label_Last()
    {
        assert!(Order_Of("v14.9") < Order_Of("v14.10"));
        assert_eq!(Order_Of("garbage"), Numbered { major: u32::MAX, minor: u32::MAX });
    }

    #[test]
    fn Test_Label_Gaps_Should_Report_A_Skipped_Minor_Version()
    {
        let labels = ["v14.25", "v14.27"].map(str::to_owned).to_vec();

        assert_eq!(Label_Gaps(&labels), vec!["v14.26".to_owned()]);
    }

    #[test]
    fn Test_Missing_Between_Should_List_Skipped_Minor_Numbers_Within_One_Major()
    {
        let before = Numbered { major: 14, minor: 25 };
        let after = Numbered { major: 14, minor: 28 };

        assert_eq!(Missing_Between(before, after), vec!["v14.26".to_owned(), "v14.27".to_owned()]);

        let across_major = Missing_Between(Numbered { major: 14, minor: 36 }, Numbered { major: 15, minor: 0 });
        assert!(across_major.is_empty());
    }

    fn Temporary_Directory_With(name: &str, files: &[&str]) -> PathBuf
    {
        let directory = std::env::temp_dir().join(format!("nomos-spec-ingest-labels-{name}"));
        std::fs::create_dir_all(&directory).expect("creates the directory");

        for file in files
        {
            std::fs::write(directory.join(file), b"").expect("writes a placeholder file");
        }

        return directory;
    }
}
