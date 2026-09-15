//! The walk every body of `nomos workflow` defers to [`super::Run`], and the one guard each
//! walking body owes it.

use nomos_rules::SourceFile;
use nomos_workflow_orchestration::{CheckBody, CorrectionBody, GateBody};
use std::path::{Path, PathBuf};

/// Every `.rs` or `.go` file under `root`, with its text and the subject its facts are filed
/// under -- a near-duplicate of `correct.rs`'s own walk rather than a shared dependency on it,
/// the identical division that module's own doc names: a composition root's own territory is
/// per group, not shared through a third module neither group's item reserved.
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

fn Read_Entry(root: &Path, path: PathBuf, pending: &mut Vec<PathBuf>, sources: &mut Vec<SourceFile>)
{
    if path.is_dir()
    {
        let skipped = path.file_name().is_some_and(|name| return name == "target" || name == ".git");

        if !skipped
        {
            pending.push(path);
        }

        return;
    }

    if path.extension().is_some_and(|extension| return extension == "rs" || extension == "go") && let Ok(text) = std::fs::read_to_string(&path)
    {
        let source = Read_Source(root, &path, text);

        sources.push(source);
    }
}

fn Read_Source(root: &Path, path: &Path, text: String) -> SourceFile
{
    use nomos_model::Subject_Of_Path;

    let relative = Relative(root, path);
    let subject = Subject_Of_Path(&relative);

    return SourceFile::New(relative, subject, text);
}

/// A path as it should be reported: relative to the tree, forward slashes.
fn Relative(root: &Path, path: &Path) -> String
{
    return path.strip_prefix(root).unwrap_or(path).display().to_string().replace('\\', "/");
}

/// `check`, with its own `sources` replaced by a real walk of its `root` -- `None` if `root` is
/// not a directory, the identical guard `gate.rs`'s and `correct.rs`'s own walks already give a
/// tree that cannot be read at all.
pub(super) fn Walked_Check(check: &CheckBody) -> Option<CheckBody>
{
    if !check.root.is_dir()
    {
        return None;
    }

    let sources = Read_Sources(&check.root);

    return Some(CheckBody::New(check.root.clone(), sources, check.selected.clone()));
}

/// `correction`, with its own `sources` replaced by a real walk of its `root` -- `None` if `root`
/// is not a directory, the identical guard [`Walked_Check`] gives `Body::Check`.
/// `Run_Correction` itself already reports `NoSourceFound` for an empty, but real, walk, so there
/// is no second empty-source guard to duplicate here the way the parent module keeps one for
/// `Body::Check`.
pub(super) fn Walked_Correction(correction: &CorrectionBody) -> Option<CorrectionBody>
{
    if !correction.root.is_dir()
    {
        return None;
    }

    let sources = Read_Sources(&correction.root);

    return Some(CorrectionBody::New(correction.root.clone(), sources, correction.commit));
}

/// `gate`, with its own `sources` replaced by a real walk of `gate.command.root` -- `None` if that
/// root is not a directory, the identical guard [`Walked_Check`] and [`Walked_Correction`] both
/// already give.
pub(super) fn Walked_Gate(gate: &GateBody) -> Option<GateBody>
{
    if !gate.command.root.is_dir()
    {
        return None;
    }

    let sources = Read_Sources(&gate.command.root);

    return Some(GateBody::New(sources, gate.command.clone()));
}
