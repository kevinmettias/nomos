//! Every file under the root the run is to judge.
//!
//! A deliberate near-duplicate of `crate::check::sources`, not a shared dependency on it:
//! that module's walk functions are `pub(super)` to `check`, and `OD-HOST-002`'s own
//! division of a composition root's territory is per group, not shared through a third
//! module neither group's item reserved. `Check_Completeness_Mirrors` needs every `.rs`
//! and `.go` file under the root — a check name claimed in one language's file can be
//! declared in the other's — so this walk stays the same shape as `check`'s own.

use super::{Path, PathBuf, SourceFile, Subject_Of_Path};

/// The Rust and Go sources under the root, or `None` if the root is not a directory.
pub(super) fn Walked(root: &Path) -> Option<Vec<SourceFile>>
{
    if !root.is_dir()
    {
        return None;
    }

    return Some(Read_Sources(root));
}

/// Every `.rs` or `.go` file under `root`, with its text and the subject its facts are
/// filed under. `target` is skipped: it holds generated source nobody authored.
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

fn Read_Entry(
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

    if path.extension().is_some_and(|extension| return extension == "rs" || extension == "go")
        && let Ok(text) = std::fs::read_to_string(&path)
    {
        let source = Read_Source(root, &path, text);
        sources.push(source);
    }
}

fn Read_Source(root: &Path, path: &Path, text: String) -> SourceFile
{
    let relative = Relative(root, path);
    let subject = Subject_Of_Path(&relative);

    return SourceFile::New(relative, subject, text);
}

/// A path as it should be reported: relative to the tree, forward slashes.
pub(super) fn Relative(root: &Path, path: &Path) -> String
{
    return path
        .strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/");
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Go_File_Should_Be_Discovered_Alongside_A_Rust_One()
    {
        let root = Fresh_Root("nomos-cli-correct-sources-go-discovery");
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
        let root = Fresh_Root("nomos-cli-correct-sources-unrelated-extension");
        std::fs::write(root.join("README.md"), "# not source\n").expect("writable");

        let sources = Read_Sources(&root);

        let _ignored = std::fs::remove_dir_all(&root);
        assert!(sources.is_empty(), "{sources:?}");
    }

    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }
}
