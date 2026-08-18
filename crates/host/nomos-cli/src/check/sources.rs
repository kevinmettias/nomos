//! Every file under the root the run is to judge.

use super::{Path, PathBuf, SourceFile, Subject_Of_Path};

/// The Rust sources under the root, or `None` if the root is not a directory.
///
/// A directory that is walked and turns out empty is not this function's decision any
/// more: `nomos_check_orchestration::CheckOutcome` is where "not a directory" and "found
/// nothing" become distinguishable typed answers, so this stays the walk and nothing else
/// -- the same division `nomos-cli::work::Published_Records` draws around the directory
/// listing `nomos_platform::FileSystem` has no port for.
pub(super) fn Walked(root: &Path) -> Option<Vec<SourceFile>>
{
    if !root.is_dir()
    {
        return None;
    }

    return Some(Read_Sources(root));
}

/// Every `.rs` file under `root`, with its text and the subject its facts are filed under.
///
/// `target` is skipped: it holds generated source that nobody authored, and judging a
/// build artifact would report findings against code the author cannot edit.
pub(super) fn Read_Sources(root: &Path) -> Vec<SourceFile>
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

/// One entry of a walked directory: queued if it is a directory worth descending into,
/// read if it is a `.rs` file, and ignored otherwise.
pub(super) fn Read_Entry(
    root: &Path,
    path: PathBuf,
    pending: &mut Vec<PathBuf>,
    sources: &mut Vec<SourceFile>,
)
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
///
/// This root files a fact under the subject and hands the same value to the rule on
/// `SourceFile::subject`, so the two cannot disagree about addressing. It is the kernel's
/// rule and not a local one, which is what keeps that agreement from being a coincidence —
/// see `OD-MODEL-001`.
pub(super) fn Read_Source(root: &Path, path: &Path, text: String) -> SourceFile
{
    let relative = Relative(root, path);
    let subject = Subject_Of_Path(&relative);

    return SourceFile::New(relative, subject, text);
}

/// A path as it should be reported: relative to the tree, forward slashes.
///
/// Forward slashes on every platform, because a finding's location appears in output that
/// gets pasted between machines, and the same file must not render two ways.
pub(super) fn Relative(root: &Path, path: &Path) -> String
{
    return path
        .strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/");
}
