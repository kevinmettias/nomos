use crate::VARIABLES;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The corpus variables a file actually reads, in [`VARIABLES`] order.
///
/// Comment lines do not count. A variable named in prose is a reference and not a use —
/// this file documents all three and gates on none of them, and so may the next one. The
/// same rule as `Test_A_Capability_Id_Should_Be_Written_In_One_Crate`, for the same reason:
/// a check that cannot tell an explanation from a declaration eventually gets satisfied by
/// deleting the explanation.
pub(crate) fn Variables_Read_By(text: &str) -> Vec<&'static str>
{
    let code: String = text
        .lines()
        .filter(|line| return !line.trim_start().starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n");

    return VARIABLES
        .iter()
        .filter(|variable| return code.contains(**variable))
        .copied()
        .collect();
}

/// Every `.rs` file under a workspace member's `tests/` that names a corpus variable,
/// repo-relative with forward slashes.
///
/// Walks the tree rather than asking cargo, because a test file is not a compilation target
/// cargo metadata will enumerate for us, and the question here is about files on disk.
pub(crate) fn Files_Naming_A_Corpus(root: &Path) -> BTreeSet<String>
{
    use nomos_contract_tests::Workspace;

    let workspace = Workspace::Load();
    let mut found = BTreeSet::new();
    for member in workspace.Members()
    {
        let tests = member.root.join("tests");
        for file in Rust_Files(&tests)
        {
            let named = Named_If_It_Reaches_A_Corpus(&file, root);

            found.extend(named);
        }
    }

    return found;
}

/// A file's repo-relative path with forward slashes, if it reads a corpus variable.
pub(crate) fn Named_If_It_Reaches_A_Corpus(file: &Path, root: &Path) -> Option<String>
{
    let text = std::fs::read_to_string(file).ok()?;
    if Variables_Read_By(&text).is_empty()
    {
        return None;
    }

    let relative = file.strip_prefix(root).ok()?;

    return Some(relative.display().to_string().replace('\\', "/"));
}

/// Every `.rs` file under a directory, recursively.
///
/// A local copy of `boundaries/common.rs`'s walk rather than a shared one. Two test binaries
/// that
/// share a helper share its failure, and these two files check different properties for
/// different reasons — the duplication is four lines and the coupling would be permanent.
pub(crate) fn Rust_Files(root: &Path) -> Vec<PathBuf>
{
    let mut found = Vec::new();
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
            Keep_Or_Descend(&entry.path(), &mut pending, &mut found);
        }
    }

    return found;
}

/// A directory to walk later, a Rust file to keep, or neither.
pub(crate) fn Keep_Or_Descend(path: &Path, pending: &mut Vec<PathBuf>, found: &mut Vec<PathBuf>)
{
    if path.is_dir()
    {
        pending.push(path.to_path_buf());
    }
    else if path.extension().is_some_and(|extension| return extension == "rs")
    {
        found.push(path.to_path_buf());
    }
}
