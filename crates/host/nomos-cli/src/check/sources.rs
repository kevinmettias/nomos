//! Every file under the root the run is to judge.

use super::{Path, Write, SourceFile, ExitCode, PathBuf, Subject_Of_Path};

/// The Rust sources under the root.
///
/// A walk that found nothing is a refusal rather than a clean result: a run that judged no
/// file renders exactly like a run that judged every file and found nothing to say.
pub(super) fn Walked(root: &Path, stderr: &mut impl Write) -> Result<Vec<SourceFile>, ExitCode>
{
    if !root.is_dir()
    {
        let _ignored = writeln!(stderr, "cannot read `{}`: not a directory", root.display());

        return Err(ExitCode::Unreadable);
    }

    let sources = Read_Sources(root);
    if sources.is_empty()
    {
        let _ignored = writeln!(
            stderr,
            "no Rust source found under `{}`, so nothing was judged.\n\
             A clean result here would mean only that the walk found nothing.",
            root.display()
        );

        return Err(ExitCode::Vacuous);
    }

    return Ok(sources);
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
