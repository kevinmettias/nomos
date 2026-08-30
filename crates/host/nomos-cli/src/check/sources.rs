//! Every file under the root the run is to judge.

use super::{Path, PathBuf, SourceFile, Subject_Of_Path};

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
    fn Test_Read_Sources_Should_Discover_A_Go_File_Alongside_A_Rust_One()
    {
        let root = Fresh_Root("nomos-cli-check-sources-go-discovery");
        std::fs::write(root.join("a.rs"), "pub fn One() {}\n").expect("writable");
        std::fs::write(root.join("main.go"), "package main\n\nfunc One() {}\n").expect("writable");

        let sources = Read_Sources(&root);

        let _ignored = std::fs::remove_dir_all(&root);
        let paths: Vec<&str> = sources.iter().map(|source| return source.path.as_str()).collect();
        assert_eq!(paths, vec!["a.rs", "main.go"], "{paths:?}");
    }

    #[test]
    fn Test_Read_Entry_Should_Not_Discover_An_Unrelated_Extension()
    {
        let root = Fresh_Root("nomos-cli-check-sources-unrelated-extension");
        std::fs::write(root.join("README.md"), "# not source\n").expect("writable");

        let sources = Read_Sources(&root);

        let _ignored = std::fs::remove_dir_all(&root);
        assert!(sources.is_empty(), "{sources:?}");
    }

    /// `Walked_Sources` wraps `Read_Sources` with one guard of its own: a root that is not a
    /// directory at all reports `None` rather than an empty list, so a caller can tell "there
    /// was nothing to read" from "there was nowhere to read".
    #[test]
    fn Test_Walked_Sources_Should_Be_None_When_The_Root_Is_Not_A_Directory()
    {
        let root = std::env::temp_dir().join("nomos-cli-check-sources-walked-sources-missing");
        let _ignored = std::fs::remove_dir_all(&root);

        assert!(Walked_Sources(&root).is_none());
    }

    /// The positive control for the guard above: a real directory reports `Some`, carrying
    /// exactly what `Read_Sources` would have found.
    #[test]
    fn Test_Walked_Sources_Should_Return_Sources_For_A_Real_Directory()
    {
        let root = Fresh_Root("nomos-cli-check-sources-walked-sources-present");
        std::fs::write(root.join("a.rs"), "pub fn One() {}\n").expect("writable");

        let sources = Walked_Sources(&root).expect("a directory returns Some");

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(sources.len(), 1);
    }

    /// `Read_Source` files a `SourceFile` under the path relative to `root` -- the same
    /// value `Relative_Path` computes -- rather than the absolute path it was read from.
    #[test]
    fn Test_Read_Source_Should_File_The_Relative_Path()
    {
        let root = Fresh_Root("nomos-cli-check-sources-read-source");
        let path = root.join("a.rs");

        let source = Read_Source(&root, &path, "pub fn One() {}\n".to_owned());

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(source.path, "a.rs");
    }

    /// Forward slashes on every platform, because a finding's location is pasted between
    /// machines and the same file must not render two ways -- the property this function
    /// exists for, checked directly against a nested path rather than only observed as a
    /// side effect of a walk.
    #[test]
    fn Test_Relative_Path_Should_Use_Forward_Slashes_On_Every_Platform()
    {
        let root = PathBuf::from("root");
        let path = root.join("nested").join("file.rs");

        assert_eq!(Relative_Path(&root, &path), "nested/file.rs");
    }

    /// The one place a walked file's own address and text come together into a
    /// `SourceFile` -- checked directly, not only observed as a side effect of a walk.
    #[test]
    fn Test_Read_Source_Should_Build_A_Source_File_Carrying_What_It_Was_Given()
    {
        let root = PathBuf::from("root");
        let path = root.join("nested").join("file.rs");

        let source = Read_Source(&root, &path, "fn Main() {}".to_owned());

        assert_eq!(source.path, "nested/file.rs");
        assert_eq!(source.text, "fn Main() {}");
    }
}

/// The Rust and Go sources under the root, or `None` if the root is not a directory.
///
/// A directory that is walked and turns out empty is not this function's decision any
/// more: `nomos_check_orchestration::CheckOutcome` is where "not a directory" and "found
/// nothing" become distinguishable typed answers, so this stays the walk and nothing else
/// -- the same division `nomos-cli::work::Published_Records` draws around the directory
/// listing `nomos_platform::FileSystem` has no port for.
pub(super) fn Walked_Sources(root: &Path) -> Option<Vec<SourceFile>>
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
/// read if it is a `.rs` or `.go` file, and ignored otherwise.
///
/// The extension check is a literal, the same as `nomos_lang_rust::RUST_EXTENSION` and
/// `nomos_lang_go::GO_EXTENSION` already state, rather than a dependency on either crate:
/// this walk decides which bytes are worth reading at all, not which registered provider
/// answers for them -- `nomos-check-orchestration::composition::Recognized_Syntax_Provider`
/// is where that second, real question is decided, over a path this function has already
/// let through.
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

    if path.extension().is_some_and(|extension| return extension == "rs" || extension == "go")
        && let Ok(text) = std::fs::read_to_string(&path)
    {
        let source = Read_Source(root, &path, text);
        sources.push(source);
    }
}

/// One source file as the rule takes it.
///
/// This root files a fact under the subject and hands the same value to the rule on
/// `SourceFile::subject`, so the two cannot disagree about addressing. It is the kernel's
/// rule and not a local one, which is what keeps that agreement from being a coincidence —
/// see `OD-MODEL-001`.
pub(super) fn Read_Source(root: &Path, path: &Path, text: String) -> SourceFile
{
    let relative = Relative_Path(root, path);
    let subject = Subject_Of_Path(&relative);

    return SourceFile::New(relative, subject, text);
}

/// A path as it should be reported: relative to the tree, forward slashes.
///
/// Forward slashes on every platform, because a finding's location appears in output that
/// gets pasted between machines, and the same file must not render two ways.
pub(super) fn Relative_Path(root: &Path, path: &Path) -> String
{
    return path
        .strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/");
}
