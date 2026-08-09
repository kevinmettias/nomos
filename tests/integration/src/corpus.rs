//! Turning a directory of files into subjects the analysis kernel can talk about.

use nomos_contracts::SubjectId;
use nomos_lang_rust::Recognition;
use nomos_model::Content_Digest;
use std::path::{Path, PathBuf};

/// One file, as the slice sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile
{
    /// Identity of the file itself.
    pub subject: SubjectId,
    /// Corpus-relative path, normalized. Kept for reporting: a failure that names a
    /// digest is a failure nobody can go and look at.
    pub path: String,
    /// The directory this file sits in, normalized. The subject of the rollup that reads
    /// it, and the reason a change to one file has a descendant at all.
    pub group: String,
    /// Identity of that directory.
    pub group_subject: SubjectId,
    pub source: String,
}

/// A corpus, walked.
#[derive(Clone, Debug)]
pub struct Corpus
{
    pub root: PathBuf,
    pub files: Vec<SourceFile>,
    /// Files that could not be read off disk. Not a parse failure and not zero files —
    /// the machine refused, and that is a third thing.
    pub unreadable: Vec<PathBuf>,
}

impl Corpus
{
    /// Every distinct group in the corpus, in a deterministic order.
    #[must_use]
    pub fn Groups(&self) -> Vec<String>
    {
        let mut groups: Vec<String> = self
            .files
            .iter()
            .map(|file| return file.group.clone())
            .collect();
        groups.sort();
        groups.dedup();

        return groups;
    }

    /// The files in one group, in corpus order.
    #[must_use]
    pub fn In_Group(&self, group: &str) -> Vec<&SourceFile>
    {
        return self
            .files
            .iter()
            .filter(|file| return file.group == group)
            .collect();
    }

    /// Replaces one file's contents, as an edit would.
    ///
    /// Returns whether the file was there to replace. A silent no-op here would make the
    /// invalidation tests assert that changing nothing invalidates nothing.
    pub fn Rewrite(&mut self, path: &str, source: &str) -> bool
    {
        for file in &mut self.files
        {
            if file.path == path
            {
                source.clone_into(&mut file.source);
                return true;
            }
        }

        return false;
    }
}

/// The identity of the subject a corpus-relative path denotes.
///
/// # Why this normalization is written here and not imported
///
/// `nomos_ledger::Subject_Of` computes the same thing by the same rules for work
/// territory. Reaching for it would couple what a *fact* is about to what a *claim* is
/// about — two vocabularies that agree today and have no reason to stay agreed. The rules
/// are duplicated deliberately and the duplication is worth converging behind one home
/// the moment a third caller appears; until then, an import would be the wrong kind of
/// agreement.
///
/// Separators are unified, `.` segments dropped, and case folded — because `Main.rs` and
/// `main.rs` are one file on the two platforms this runs on, and treating them as two
/// subjects would let one edit invalidate neither.
#[must_use]
pub fn Subject_Of_Path(path: &str) -> SubjectId
{
    return SubjectId::From_Digest(Content_Digest(Normalize(path).as_bytes()));
}

fn Normalize(path: &str) -> String
{
    return path
        .trim()
        .replace('\\', "/")
        .split('/')
        .filter(|segment| return !segment.is_empty() && *segment != ".")
        .collect::<Vec<&str>>()
        .join("/")
        .to_lowercase();
}

/// Directories that are not somebody's source.
const NOT_SOURCE: &[&str] = &["target", ".git"];

/// Walks a tree into subjects.
///
/// Sorted, so that a failure names the same file on two machines and every count below is
/// taken over the same set in the same order. Recognition decides what is read, rather
/// than a second extension check written here — two answers to "does this provider read
/// this file" is one answer too many.
#[must_use]
pub fn Walk(root: &Path) -> Corpus
{
    let mut paths = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop()
    {
        let Ok(entries) = std::fs::read_dir(&directory)
        else
        {
            continue;
        };

        for entry in entries.flatten()
        {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();

            if path.is_dir()
            {
                if !NOT_SOURCE.contains(&name.as_ref())
                {
                    pending.push(path);
                }
                continue;
            }

            if Recognition::Of_Path(&name) == Recognition::Recognized
            {
                paths.push(path);
            }
        }
    }

    paths.sort();

    let mut files = Vec::new();
    let mut unreadable = Vec::new();

    for path in paths
    {
        let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
        let relative = Normalize(&relative);
        let group = relative
            .rsplit_once('/')
            .map_or_else(|| return String::new(), |(directory, _)| return directory.to_owned());

        let Ok(source) = std::fs::read_to_string(&path)
        else
        {
            unreadable.push(path);
            continue;
        };

        files.push(SourceFile {
            subject: Subject_Of_Path(&relative),
            path: relative,
            group_subject: Subject_Of_Path(&group),
            group,
            source,
        });
    }

    return Corpus {
        root: root.to_path_buf(),
        files,
        unreadable,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Spellings_Of_One_Path_Should_Be_One_Subject()
    {
        let canonical = Subject_Of_Path("alpha/one.rs");

        for spelling in ["./alpha/one.rs", "alpha\\one.rs", "alpha//one.rs", "Alpha/One.rs"]
        {
            assert_eq!(Subject_Of_Path(spelling), canonical, "`{spelling}`");
        }
    }

    /// The negative control. If normalization collapsed everything, every file in the
    /// corpus would be one subject and a single edit would invalidate the world.
    #[test]
    fn Test_Different_Paths_Should_Be_Different_Subjects()
    {
        assert_ne!(Subject_Of_Path("alpha/one.rs"), Subject_Of_Path("alpha/two.rs"));
        assert_ne!(Subject_Of_Path("alpha/one.rs"), Subject_Of_Path("beta/one.rs"));
        assert_ne!(Subject_Of_Path("alpha"), Subject_Of_Path("alpha/one.rs"));
    }
}
