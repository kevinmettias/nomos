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

    /// Removes and recreates `root` under the system temp directory, so a test starts
    /// from a clean, empty tree regardless of what an earlier run left behind.
    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }

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

    #[test]
    fn Test_An_Unrelated_Extension_Should_Not_Be_Discovered()
    {
        let root = Fresh_Root("nomos-api-sources-unrelated-extension");
        std::fs::write(root.join("README.md"), "# not source\n").expect("writable");

        let sources = Read_Sources(&root);

        let _ignored = std::fs::remove_dir_all(&root);
        assert!(sources.is_empty(), "{sources:?}");
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
/// if it is a `.rs` or `.go` file, and ignored otherwise.
///
/// The extension check is a literal, the same as `nomos_lang_rust::RUST_EXTENSION` and
/// `nomos_lang_go::GO_EXTENSION` already state, rather than a dependency on either crate:
/// this walk decides which bytes are worth reading at all, not which registered provider
/// answers for them -- `nomos-check-orchestration::composition::Recognized_Syntax_Provider`
/// is where that second, real question is decided, over a path this function has already
/// let through.
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

    if path.extension().is_some_and(|extension| return extension == "rs" || extension == "go")
        && let Ok(text) = std::fs::read_to_string(&path)
    {
        let source = Read_Source(root, &path, text);
        sources.push(source);
    }
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
