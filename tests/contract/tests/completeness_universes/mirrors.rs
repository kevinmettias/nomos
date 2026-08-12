//! A named mirror has to exist.

use crate::table::{Standing, UNIVERSES};
use nomos_contract_tests::Workspace;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// A row can claim a mirror that was renamed or deleted, and the table would still read as
/// though the universe were covered. That is the same defect one level up.
#[test]
fn Test_Every_Named_Mirror_Should_Exist_In_The_Source()
{
    let functions = Function_Names();
    assert!(
        !functions.is_empty(),
        "no functions were found, so this assertion would pass having read nothing"
    );

    let mut missing = Vec::new();
    for universe in UNIVERSES
    {
        if let Standing::Mirrored { by } = universe.standing
            && !functions.contains(by)
        {
            missing.push((universe.name, by));
        }
    }

    assert!(
        missing.is_empty(),
        "these rows name a mirror that does not exist: {missing:#?}.\n\
         Either the test was renamed, in which case update the row, or it was deleted, in \
         which case the universe is unmirrored and UNMIRRORED_TOTAL must rise."
    );
}

/// Every function name declared anywhere in the workspace's own sources.
fn Function_Names() -> BTreeSet<String>
{
    let workspace = Workspace::Load();
    let mut names = BTreeSet::new();

    for member in workspace.Members()
    {
        for directory in ["src", "tests"]
        {
            let source_root = member.root.join(directory);
            if !source_root.is_dir()
            {
                continue;
            }

            let declared = Function_Names_Under(&source_root);
            names.extend(declared);
        }
    }

    return names;
}

/// The function names declared under one source root.
fn Function_Names_Under(root: &Path) -> BTreeSet<String>
{
    let mut names = BTreeSet::new();

    for file in Source_Files(root)
    {
        let Ok(text) = std::fs::read_to_string(&file)
        else
        {
            continue;
        };

        for line in text.lines()
        {
            if let Some(rest) = line.trim().strip_prefix("fn ")
                && let Some((name, _)) = rest.split_once('(')
            {
                names.insert(name.trim().to_owned());
            }
        }
    }

    return names;
}

fn Source_Files(root: &Path) -> Vec<PathBuf>
{
    let mut files = Vec::new();
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
            Keep_Or_Descend(&entry.path(), &mut pending, &mut files);
        }
    }

    return files;
}

/// A directory to walk later, a Rust file to keep, or neither.
fn Keep_Or_Descend(path: &Path, pending: &mut Vec<PathBuf>, files: &mut Vec<PathBuf>)
{
    if path.is_dir()
    {
        pending.push(path.to_path_buf());
    }
    else if path.extension().is_some_and(|extension| extension == "rs")
    {
        files.push(path.to_path_buf());
    }
}
