//! Every `.rs` file under a crate's `src/` must be reachable from its root by following
//! `mod` declarations.

use crate::bands::{Declared_Modules, Source_Files};
use std::collections::BTreeSet;
use std::path::Path;

/// Adopted from the sibling workspace, where it found 95 orphaned files across seven
/// crates — including a complete, documented, four-test module that had never once
/// compiled, and a test file that had never run a single assertion.
///
/// The compiler says nothing about a file no `mod` names. It never parses it. So the
/// file does not type-check, its tests do not run, its lints do not fire and its denies
/// do not apply — while reading, to every human, as finished work. An orphan is strictly
/// worse than a missing file: a missing file is an absence, and an orphan is a false
/// claim of coverage.
#[test]
fn Test_Every_Source_File_Should_Be_Reachable()
{
    use nomos_contract_tests::Workspace;

    let workspace = Workspace::Load();
    let mut orphans = Vec::new();
    for member in workspace.Members()
    {
        let found = Orphans_Under(&member.root);

        orphans.extend(found);
    }

    assert!(
        orphans.is_empty(),
        "these files are not reachable from any crate root: {orphans:#?}.\n\
         rustc never parses an undeclared file, so its tests do not run and its lints do \
         not fire, while it reads as finished work. Declare it or delete it."
    );
}

/// Every file under one crate's `src/` that no `mod` declaration names.
fn Orphans_Under(root: &Path) -> Vec<String>
{
    let source_root = root.join("src");
    let declared = Declared_Modules(&source_root);
    let mut orphans = Vec::new();
    for file in Source_Files(&source_root)
    {
        let orphan = Orphan(&file, root, &declared);

        orphans.extend(orphan);
    }

    return orphans;
}

/// The file's path relative to its crate, if nothing declares it.
///
/// Crate roots and module roots are reached by cargo and by their parent directory's
/// declaration respectively, not by a `mod` naming their stem.
fn Orphan(file: &Path, root: &Path, declared: &BTreeSet<String>) -> Option<String>
{
    let stem = file.file_stem().and_then(|stem| stem.to_str())?;
    if matches!(stem, "lib" | "main" | "mod") || declared.contains(stem)
    {
        return None;
    }

    return Some(file.strip_prefix(root).unwrap_or(file).display().to_string());
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Genuinely_Undeclared_File_Should_Still_Be_An_Orphan()
    {
        let declared: BTreeSet<String> = ["applied", "change"].into_iter().map(str::to_owned).collect();
        let stray = Path::new("crates/example/src/stray.rs");

        assert_eq!(Orphan(stray, Path::new("crates/example"), &declared), Some("src/stray.rs".to_owned()));
    }

    #[test]
    fn Test_A_Declared_File_Should_Not_Be_An_Orphan()
    {
        let declared: BTreeSet<String> = ["applied", "change"].into_iter().map(str::to_owned).collect();
        let known = Path::new("crates/example/src/change.rs");

        assert_eq!(Orphan(known, Path::new("crates/example"), &declared), None);
    }
}
