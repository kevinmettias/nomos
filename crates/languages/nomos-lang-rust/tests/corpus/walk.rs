//! Which files of a real tree this provider is asked about.
//!
//! One walk, used twice — once to collect what the provider reads and once to count what it
//! skips. Shared deliberately, so a directory exclusion cannot apply to one of them and not
//! the other.

use nomos_lang_rust::Recognition;
use std::path::{Path, PathBuf};

/// Directories that are not somebody's source.
///
/// `target` is compiler output — reading it would inflate every count here with generated
/// code and measure this provider against rustc's formatting rather than against a person's.
/// `.git` is object storage that happens to sit in the tree.
const NOT_SOURCE: &[&str] = &["target", ".git"];

/// Every file under a root that is not inside a directory nobody's source lives in.
pub(crate) fn Each_File(root: &Path, mut visit: impl FnMut(&Path, &str))
{
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
            Visit(&entry, &mut pending, &mut visit);
        }
    }
}

/// One directory entry: a source directory is queued, and a file is handed to the caller.
fn Visit(
    entry: &std::fs::DirEntry,
    pending: &mut Vec<PathBuf>,
    visit: &mut impl FnMut(&Path, &str),
)
{
    let path = entry.path();
    let name = entry.file_name();
    let name = name.to_string_lossy();

    if !path.is_dir()
    {
        visit(&path, &name);

        return;
    }
    if !NOT_SOURCE.contains(&name.as_ref())
    {
        pending.push(path);
    }
}

/// Every recognized file under a root, in a deterministic order.
///
/// Sorted rather than left in directory order, so that a failure names the same file on
/// two machines and a report of "the first ten failures" is the same ten.
pub(crate) fn Rust_Files(root: &Path) -> Vec<PathBuf>
{
    let mut found = Vec::new();

    // Recognition decides, rather than a second extension check written here. Two answers
    // to "does this provider read this file" is one answer too many.
    Each_File(root, |path, name| {
        if Recognition::Of_Path(name) == Recognition::Recognized
        {
            found.push(path.to_path_buf());
        }
    });
    found.sort();

    return found;
}
