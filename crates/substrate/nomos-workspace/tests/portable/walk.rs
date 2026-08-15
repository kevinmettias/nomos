//! Reading a real tree off the disk as the pairs a change set is built from.
//!
//! Here rather than beside the tests because it is about the filesystem and not about the
//! workspace: what counts as a source file, what a workspace-relative path is, and the guard
//! against a walk that found a directory and almost nothing in it.

// PROBE


use std::path::{Path, PathBuf};

const NOT_SOURCE: &[&str] = &["target", ".git"];

/// Every Rust file under a root, as workspace-relative paths and contents.
///
/// Sorted, so a failure names the same file on two machines and the permutation test has a
/// stable baseline to permute away from.
pub(crate) fn Corpus(root: &Path) -> Vec<(String, String)>
{
    let mut paths = Rust_Files_Under(root);
    paths.sort();

    let mut members = Vec::new();
    for path in paths
    {
        // Read as bytes and render lossily rather than requiring UTF-8. A file this
        // workspace cannot decode is still a member of it, and skipping it would make the
        // snapshot describe a tree that is missing files nobody was told about.
        let Ok(bytes) = std::fs::read(&path)
        else
        {
            continue;
        };
        let relative = Relative_To(root, &path);
        members.push((relative, String::from_utf8_lossy(&bytes).into_owned()));
    }

    return members;
}

/// Every Rust file under a root, in whatever order the walk found them.
fn Rust_Files_Under(root: &Path) -> Vec<PathBuf>
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
            Visit(&entry.path(), &mut pending, &mut paths);
        }
    }

    return paths;
}

/// One entry: a source directory to descend into later, a Rust file to keep, or neither.
fn Visit(path: &Path, pending: &mut Vec<PathBuf>, paths: &mut Vec<PathBuf>)
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
    else if !is_directory && path.extension().is_some_and(|extension| return extension == "rs")
    {
        paths.push(path.to_path_buf());
    }
}

/// A path under the root, as the workspace-relative string a snapshot is allowed to record.
fn Relative_To(root: &Path, path: &Path) -> String
{
    return path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
}

/// The guard against a walk that found a directory but almost nothing in it.
pub(crate) fn Assert_This_Is_That_Corpus(members: &[(String, String)], root: &Path)
{
    assert!(
        members.len() >= 5_000,
        "found {} Rust files under {}, which is not this corpus. Every assertion below \
         iterates over this set, so a truncated walk makes all of them pass having read \
         almost nothing",
        members.len(),
        root.display()
    );
}
