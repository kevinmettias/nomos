//! Turning a directory of files into subjects the analysis kernel can talk about.

use nomos_contracts::SubjectId;
use nomos_lang_rust::Recognition;
use nomos_model::Normalize_Path;
use std::path::{Path, PathBuf};

/// The identity of the subject a corpus-relative path denotes.
///
/// Re-exported rather than written here. This module used to carry its own copy, with a
/// doc comment saying the duplication was "worth converging behind one home the moment a
/// third caller appears" — `OD-RULES-001` made that third caller and `OD-MODEL-001` did
/// the converging. The re-export is what keeps `crate::corpus::Subject_Of_Path` the name
/// the rest of the harness already spells.
pub use nomos_model::Subject_Of_Path;

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
    let mut paths = Recognized_Files_Under(root);
    paths.sort();

    let mut files = Vec::new();
    let mut unreadable = Vec::new();
    for path in paths
    {
        let read = Read_One(root, &path);
        let Some(file) = read
        else
        {
            unreadable.push(path);
            continue;
        };
        files.push(file);
    }

    return Corpus {
        root: root.to_path_buf(),
        files,
        unreadable,
    };
}

/// Every file the providers recognize under a root, in whatever order the walk found them.
fn Recognized_Files_Under(root: &Path) -> Vec<PathBuf>
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
            Sort_One_Entry(&entry.path(), &mut pending, &mut paths);
        }
    }

    return paths;
}

/// A source directory to descend into later, a recognized file to keep, or neither.
///
/// Recognition decides what is read, rather than a second extension check written here — two
/// answers to "does this provider read this file" is one answer too many.
fn Sort_One_Entry(path: &Path, pending: &mut Vec<PathBuf>, paths: &mut Vec<PathBuf>)
{
    let Some(name) = path.file_name()
    else
    {
        return;
    };
    let name = name.to_string_lossy();
    let is_directory = path.is_dir();
    if is_directory && !NOT_SOURCE.contains(&name.as_ref())
    {
        pending.push(path.to_path_buf());
    }
    else if !is_directory && Recognition::Of_Path(&name) == Recognition::Recognized
    {
        paths.push(path.to_path_buf());
    }
}

/// One file as a subject, or nothing where this machine cannot read it.
fn Read_One(root: &Path, path: &Path) -> Option<SourceFile>
{
    let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
    let relative = Normalize_Path(&relative);
    let group = relative
        .rsplit_once('/')
        .map_or_else(|| return String::new(), |(directory, _)| return directory.to_owned());
    let source = std::fs::read_to_string(path).ok()?;

    return Some(SourceFile {
        subject: Subject_Of_Path(&relative),
        path: relative,
        group_subject: Subject_Of_Path(&group),
        group,
        source,
    });
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The spelling assertions moved to the kernel with the rule, and repeating them here
    /// would be the duplication this module just gave up in a second costume. What is left
    /// is the one thing only this side can say: which of the two subject rules a *fact*
    /// takes.
    ///
    /// The ledger folds `docs/records/<id>-<slug>.md` onto `docs/records/<id>`, so that an
    /// item can reserve a record before the file exists. Addressing a fact that way would
    /// file a record and its identifier under one subject, and an edit to one record would
    /// invalidate facts about another. `OD-MODEL-001` is why the two rules stayed
    /// distinct; this is the assertion that the harness took the right one.
    #[test]
    fn Test_A_Fact_About_A_Record_File_Should_Be_About_That_File()
    {
        assert_ne!(
            Subject_Of_Path("docs/records/od-model-001-a-slug.md"),
            Subject_Of_Path("docs/records/od-model-001")
        );
    }
}
