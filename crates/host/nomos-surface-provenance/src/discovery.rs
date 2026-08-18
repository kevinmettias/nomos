//! Which crates this report can check: every `tests/contract/surface/<crate>.txt`.
//!
//! `nomos-check`'s own composition root gives the reason this reads the filesystem
//! directly rather than through a port: "no `nomos_platform::FileSystem`
//! directory-listing port exists" — the same exception, at the same layer, for the same
//! kind of call.

use std::path::{Path, PathBuf};

/// `tests/contract/surface`, under a repository root.
pub(crate) fn Snapshot_Directory(root: &Path) -> PathBuf
{
    return root.join("tests").join("contract").join("surface");
}

/// Every crate with a snapshot file, named by that file's stem.
///
/// # Errors
///
/// Returns a message when the directory cannot be read at all — a repository root that
/// is not this repository, or is not checked out.
pub(crate) fn Every_Snapshotted_Crate(root: &Path) -> Result<Vec<String>, String>
{
    let directory = Snapshot_Directory(root);
    let entries = std::fs::read_dir(&directory).map_err(|error| {
        return format!("cannot read {}: {error}", directory.display());
    })?;

    let mut names = Vec::new();
    for entry in entries
    {
        let entry = entry.map_err(|error| format!("cannot read an entry under {}: {error}", directory.display()))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("txt")
        {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
        {
            names.push(stem.to_owned());
        }
    }

    names.sort();

    return Ok(names);
}

/// The snapshot path for one crate, relative to a repository root — the same path the
/// git queries scope themselves to.
pub(crate) fn Snapshot_Path(krate: &str) -> String
{
    return format!("tests/contract/surface/{krate}.txt");
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_This_Repositorys_Own_Surface_Directory_Yields_Its_Crates()
    {
        // Reads this repository's own checkout — not its git history, which is the part
        // `.github/workflows/gate.yml` cannot promise is complete. Listing a directory
        // that is already present in any checkout, shallow or not, is safe here.
        let root = Repository_Root();

        let crates = Every_Snapshotted_Crate(&root).expect("this repository has the directory");

        assert!(crates.contains(&"nomos-model".to_owned()));
        assert!(crates.iter().all(|name| !name.is_empty()));
    }

    #[test]
    fn Test_A_Missing_Directory_Refuses_Rather_Than_Reporting_Empty()
    {
        let result = Every_Snapshotted_Crate(Path::new("no-such-directory-at-all"));

        assert!(result.is_err());
    }

    #[test]
    fn Test_The_Snapshot_Path_Is_Repository_Relative()
    {
        assert_eq!(Snapshot_Path("nomos-model"), "tests/contract/surface/nomos-model.txt");
    }

    fn Repository_Root() -> PathBuf
    {
        // CARGO_MANIFEST_DIR is crates/host/nomos-surface-provenance; the repository root
        // is three levels up.
        return Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..");
    }
}
