//! Every file under the root this server judges.
//!
//! A deliberate twin of `nomos-cli::check::sources`: that module's own functions are
//! `pub(super)`, so a second composition root that also calls `nomos_check_orchestration::
//! Run` carries its own copy of the identical walk rather than reaching into another
//! crate's private module. `nomos-correction-orchestration`'s own `run.rs` tests build a
//! third, smaller copy of the same shape for the identical reason.

use nomos_model::Subject_Of_Path;
use nomos_rules::SourceFile;
use std::path::{Path, PathBuf};

/// The Rust and Go sources under `root`, or `None` if `root` is not a directory.
///
/// A directory that is walked and turns out empty is not this function's decision:
/// `nomos_check_orchestration::CheckOutcome` is where "not a directory" and "found nothing"
/// become distinguishable typed answers.
pub(crate) fn Walked_Sources(root: &Path) -> Option<Vec<SourceFile>>
{
    if !root.is_dir()
    {
        return None;
    }

    return Some(Read_Sources(root));
}

/// Every `.rs` or `.go` file under `root`, with its text and the subject its facts are
/// filed under.
///
/// `target` and `.git` are skipped: the first holds generated source nobody authored, and
/// the second is not source at all.
pub(crate) fn Read_Sources(root: &Path) -> Vec<SourceFile>
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
/// read if it is a recognized source file, and ignored otherwise.
fn Read_Entry(root: &Path, path: PathBuf, pending: &mut Vec<PathBuf>, sources: &mut Vec<SourceFile>)
{
    if path.is_dir()
    {
        let skipped = path.file_name().is_some_and(|name| return name == "target" || name == ".git") || Is_A_Nested_Git_Worktree(&path);

        if !skipped
        {
            pending.push(path);
        }

        return;
    }

    if path.extension().is_some_and(|extension| return extension == "rs" || extension == "go")
        && let Ok(text) = std::fs::read_to_string(&path)
    {
        sources.push(Read_Source(root, &path, text));
    }
}

/// Whether `path` is itself a git worktree's own root — a directory whose immediate `.git`
/// entry is a file (naming another repository's own `.git/worktrees/<name>` directory)
/// rather than the ordinary `.git` directory a real checkout has. A linked worktree checked
/// out under an already-walked root is a second copy of a tree this walk must not descend
/// into, the same reason `target` already is — `P67-SELF-CHECK-WALK-CROSSES-NESTED-
/// WORKTREE-BOUNDARY-2` measured exactly this against a leftover build-agent worktree left
/// under `.claude/worktrees/`.
fn Is_A_Nested_Git_Worktree(path: &Path) -> bool
{
    return path.join(".git").is_file();
}

/// One source file as `nomos_check_orchestration::Run` takes it.
///
/// This root files a fact under the subject and hands the same value on
/// `SourceFile::subject`, so the two cannot disagree about addressing -- the kernel's rule,
/// not a local one. `OD-MODEL-001`.
pub(crate) fn Read_Source(root: &Path, path: &Path, text: String) -> SourceFile
{
    let relative = Relative_Path(root, path);
    let subject = Subject_Of_Path(&relative);

    return SourceFile::New(relative, subject, text);
}

/// A path as it should be reported: relative to the tree, forward slashes.
///
/// Forward slashes on every platform, because a finding's location appears in output that
/// gets pasted between machines, and the same file must not render two ways.
pub(crate) fn Relative_Path(root: &Path, path: &Path) -> String
{
    return path.strip_prefix(root).unwrap_or(path).display().to_string().replace('\\', "/");
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Walked_Sources_Should_Be_None_When_The_Root_Is_Not_A_Directory()
    {
        let root = std::env::temp_dir().join("nomos-lsp-sources-walked-sources-missing");
        let _ignored = std::fs::remove_dir_all(&root);

        assert!(Walked_Sources(&root).is_none());
    }

    #[test]
    fn Test_Walked_Sources_Should_Return_Sources_For_A_Real_Directory()
    {
        let root = Fresh_Root("nomos-lsp-sources-walked-sources-present");
        std::fs::write(root.join("a.rs"), "pub fn One() {}\n").expect("writable");

        let sources = Walked_Sources(&root).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn Test_Read_Sources_Should_Skip_Target_And_Git_Directories()
    {
        let root = Fresh_Root("nomos-lsp-sources-skip-generated");
        std::fs::create_dir_all(root.join("target")).expect("writable");
        std::fs::write(root.join("target").join("built.rs"), "pub fn Built() {}\n").expect("writable");
        std::fs::write(root.join("a.rs"), "pub fn One() {}\n").expect("writable");

        let sources = Read_Sources(&root);

        let _ignored = std::fs::remove_dir_all(&root);
        let paths: Vec<&str> = sources.iter().map(|source| return source.path.as_str()).collect();
        assert_eq!(paths, vec!["a.rs"], "{paths:?}");
    }

    /// `P67-SELF-CHECK-WALK-CROSSES-NESTED-WORKTREE-BOUNDARY-2`: a subdirectory that is
    /// itself a git worktree's own root -- a `.git` *file*, not a `.git` directory --
    /// must not be descended into, the same as `target` already is not.
    #[test]
    fn Test_Read_Sources_Should_Not_Descend_Into_A_Nested_Git_Worktree()
    {
        let root = Fresh_Root("nomos-lsp-sources-nested-worktree");
        std::fs::write(root.join("a.rs"), "pub fn One() {}\n").expect("writable");
        let worktree = root.join("worktree");
        std::fs::create_dir_all(&worktree).expect("writable");
        std::fs::write(worktree.join(".git"), "gitdir: /elsewhere/.git/worktrees/example\n").expect("writable");
        std::fs::write(worktree.join("stale.rs"), "pub fn Stale() {}\n").expect("writable");

        let sources = Read_Sources(&root);

        let _ignored = std::fs::remove_dir_all(&root);
        let paths: Vec<&str> = sources.iter().map(|source| return source.path.as_str()).collect();
        assert_eq!(paths, vec!["a.rs"], "{paths:?}");
    }

    #[test]
    fn Test_Relative_Path_Should_Use_Forward_Slashes_On_Every_Platform()
    {
        let root = PathBuf::from("root");
        let path = root.join("nested").join("file.rs");

        assert_eq!(Relative_Path(&root, &path), "nested/file.rs");
    }

    #[test]
    fn Test_Read_Source_Should_File_The_Relative_Path()
    {
        let root = Fresh_Root("nomos-lsp-sources-read-source");
        let path = root.join("a.rs");

        let source = Read_Source(&root, &path, "pub fn One() {}\n".to_owned());

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(source.path, "a.rs");
    }

    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }
}
