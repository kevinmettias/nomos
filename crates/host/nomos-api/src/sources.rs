//! Every file under a root this crate is asked to judge.
//!
//! A deliberate twin of `crates/host/nomos-cli/src/gate/sources.rs`, not a shared dependency
//! of it -- the same "additive and unwired" separation that file's own doc keeps between it
//! and `check::sources`. Duplicated rather than factored out because a walk is a
//! composition-root concern (`OD-HOST-002`'s family 3), and this crate is a second,
//! independent composition root, not a caller of `nomos-cli`'s.

use nomos_rules::SourceFile;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Go_File_Should_Be_Discovered_Alongside_A_Rust_One()
    {
        let root = Fresh_Root("nomos-api-sources-go-discovery");
        std::fs::write(root.join("a.rs"), "pub fn One() {}\n").expect("writable");
        std::fs::write(root.join("main.go"), "package main\n\nfunc One() {}\n").expect("writable");

        let sources = Read_Sources(&root);

        let _ignored = std::fs::remove_dir_all(&root);
        let paths: Vec<&str> = sources.iter().map(|source| return source.path.as_str()).collect();
        assert_eq!(paths, vec!["a.rs", "main.go"], "{paths:?}");
    }

    /// `P67-SELF-CHECK-WALK-CROSSES-NESTED-WORKTREE-BOUNDARY-2`: a subdirectory that is
    /// itself a git worktree's own root -- a `.git` *file*, not a `.git` directory --
    /// must not be descended into, the same as `target` already is not.
    #[test]
    fn Test_A_Nested_Git_Worktree_Should_Not_Be_Descended_Into()
    {
        let root = Fresh_Root("nomos-api-sources-nested-worktree");
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
    fn Test_An_Unrelated_Extension_Should_Not_Be_Discovered()
    {
        let root = Fresh_Root("nomos-api-sources-unrelated-extension");
        std::fs::write(root.join("README.md"), "# not source\n").expect("writable");

        let sources = Read_Sources(&root);

        let _ignored = std::fs::remove_dir_all(&root);
        assert!(sources.is_empty(), "{sources:?}");
    }

    /// The population `check-script-discipline`'s own rules judge -- a shebang script and a
    /// `standards.json`-forbidden extension -- must actually reach a walk, or those rules
    /// report clean regardless of what either file does.
    #[test]
    fn Test_A_Shebang_Script_And_A_Forbidden_Script_Extension_Should_Be_Discovered()
    {
        let root = Fresh_Root("nomos-api-sources-script-discovery");
        std::fs::write(root.join("deploy.sh"), "#!/bin/bash\n# deploys\nset -euo pipefail\n").expect("writable");
        std::fs::write(root.join("tool.ps1"), "Write-Host 'hi'\n").expect("writable");

        let sources = Read_Sources(&root);

        let _ignored = std::fs::remove_dir_all(&root);
        let paths: Vec<&str> = sources.iter().map(|source| return source.path.as_str()).collect();
        assert_eq!(paths, vec!["deploy.sh", "tool.ps1"], "{paths:?}");
    }

    /// `Walked_Sources` is `Read_Sources` plus the one judgement call the two share: whether
    /// `root` is even a directory worth walking.
    #[test]
    fn Test_Walked_Sources_Should_Discover_Real_Files_And_Return_None_For_A_Non_Directory()
    {
        let root = Fresh_Root("nomos-api-sources-walked-sources");
        std::fs::write(root.join("a.rs"), "pub fn One() {}\n").expect("writable");

        let discovered = Walked_Sources(&root).expect("root is a real directory");
        let not_a_directory = Walked_Sources(&root.join("a.rs"));

        let _ignored = std::fs::remove_dir_all(&root);

        assert_eq!(discovered.len(), 1, "{discovered:?}");
        assert!(not_a_directory.is_none(), "{not_a_directory:?}");
    }

    /// Removes and recreates `root` under the system temp directory, so a test starts
    /// from a clean, empty tree regardless of what an earlier run left behind.
    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }
}

/// The Rust and Go sources under `root`, or `None` if `root` is not a directory.
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
/// if it is a recognized source or script file, and ignored otherwise.
///
/// The extension check is a literal, the same as the `RUST_EXTENSION` and `GO_EXTENSION`
/// constants nomos-lang-rust and nomos-lang-go each already state, rather than a dependency on
/// either crate: this walk decides which bytes are worth reading at all, not which registered
/// provider answers for them -- `nomos-check-orchestration::composition::Recognized_Syntax_Provider`
/// is where that second, real question is decided, over a path this function has already let
/// through.
fn Read_Entry(root: &Path, path: PathBuf, pending: &mut Vec<PathBuf>, sources: &mut Vec<SourceFile>)
{
    if path.is_dir()
    {
        let skipped = path
            .file_name()
            .is_some_and(|name| return name == "target" || name == ".git")
            || Is_A_Nested_Git_Worktree(&path);

        if !skipped
        {
            pending.push(path);
        }

        return;
    }

    if path.extension().is_some_and(Is_Recognized_Extension) && let Ok(text) = std::fs::read_to_string(&path)
    {
        let source = Read_Source(root, &path, text);
        sources.push(source);
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

/// Rust, Go, or one of `check-script-discipline`'s own script languages -- `standards.json`'s
/// `forbidden_extensions` (`.ps1`, `.psm1`, `.bat`, `.cmd`, `.sh`), the same set a real
/// shebang script in this repository would carry. Without these five, `scripts-use-a-
/// portable-shebang`, `a-script-declares-its-purpose`, `executed-scripts-set-nounset` and
/// `declared-tooling-language-for-scripts` judge a population this walk never collects, so
/// all four report clean regardless of what a script under the tree actually does.
fn Is_Recognized_Extension(extension: &std::ffi::OsStr) -> bool
{
    const SCRIPT_EXTENSIONS: [&str; 5] = ["sh", "ps1", "psm1", "bat", "cmd"];

    return extension == "rs"
        || extension == "go"
        || extension.to_str().is_some_and(|extension| return SCRIPT_EXTENSIONS.contains(&extension));
}

/// One source file as the rule takes it.
fn Read_Source(root: &Path, path: &Path, text: String) -> SourceFile
{
    use nomos_model::Subject_Of_Path;

    let relative = Relative_Path(root, path);
    let subject = Subject_Of_Path(&relative);

    return SourceFile::New(relative, subject, text);
}

/// A path as it should be reported: relative to the tree, forward slashes.
fn Relative_Path(root: &Path, path: &Path) -> String
{
    return path
        .strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/");
}
