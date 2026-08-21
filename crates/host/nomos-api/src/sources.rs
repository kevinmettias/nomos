//! Every file under a root this crate is asked to judge.
//!
//! A deliberate twin of `crates/host/nomos-cli/src/gate/sources.rs`, not a shared dependency
//! of it -- the same "additive and unwired" separation that file's own doc keeps between it
//! and `check::sources`. Duplicated rather than factored out because a walk is a
//! composition-root concern (`OD-HOST-002`'s family 3), and this crate is a second,
//! independent composition root, not a caller of `nomos-cli`'s.

use nomos_model::Subject_Of_Path;
use nomos_rules::SourceFile;
use std::path::{Path, PathBuf};

/// The Rust sources under `root`, or `None` if `root` is not a directory.
pub(crate) fn Walked(root: &Path) -> Option<Vec<SourceFile>>
{
    if !root.is_dir()
    {
        return None;
    }

    return Some(Read_Sources(root));
}

/// Every `.rs` file under `root`, with its text and the subject its facts are filed under.
///
/// `target` and `.git` are skipped: the former holds generated source nobody authored, the
/// latter is not source at all.
fn Read_Sources(root: &Path) -> Vec<SourceFile>
{
    let mut sources = Vec::new();
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
            Read_Entry(root, path, &mut pending, &mut sources);
        }
    }

    sources.sort_by(|left, right| return left.path.cmp(&right.path));
    return sources;
}

/// One entry of a walked directory: queued if it is a directory worth descending into, read
/// if it is a `.rs` file, and ignored otherwise.
fn Read_Entry(root: &Path, path: PathBuf, pending: &mut Vec<PathBuf>, sources: &mut Vec<SourceFile>)
{
    if path.is_dir()
    {
        let skipped = path
            .file_name()
            .is_some_and(|name| return name == "target" || name == ".git");

        if !skipped
        {
            pending.push(path);
        }

        return;
    }

    if path.extension().is_some_and(|extension| return extension == "rs")
        && let Ok(text) = std::fs::read_to_string(&path)
    {
        let source = Read_Source(root, &path, text);
        sources.push(source);
    }
}

/// One `.rs` file as the rule takes it.
fn Read_Source(root: &Path, path: &Path, text: String) -> SourceFile
{
    let relative = Relative(root, path);
    let subject = Subject_Of_Path(&relative);

    return SourceFile::New(relative, subject, text);
}

/// A path as it should be reported: relative to the tree, forward slashes.
fn Relative(root: &Path, path: &Path) -> String
{
    return path
        .strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/");
}
