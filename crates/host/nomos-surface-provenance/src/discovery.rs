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
        if let Some(stem) = Snapshot_Stem(&entry.path())
        {
            names.push(stem);
        }
    }

    names.sort();

    return Ok(names);
}

/// The crate name a `tests/contract/surface/` entry names, if it is a `.txt` snapshot.
fn Snapshot_Stem(path: &Path) -> Option<String>
{
    if path.extension().and_then(|extension| extension.to_str()) != Some("txt")
    {
        return None;
    }

    return path.file_stem().and_then(|stem| stem.to_str()).map(str::to_owned);
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
    fn Test_Every_Snapshotted_Crate_Should_Refuse_A_Missing_Directory_Rather_Than_Reporting_Empty()
    {
        let missing_root = Path::new("no-such-directory-at-all");
        let result = Every_Snapshotted_Crate(missing_root);

        let error = result.expect_err("a missing directory must refuse rather than report empty");
        let expected_directory = Snapshot_Directory(missing_root);

        // Names both the message shape this function actually produces and the exact
        // directory it tried to read -- a malformed input, an out-of-range index, or a
        // typo in the fixture would all still be `is_err()`, but none of them would name
        // this directory under this message.
        assert!(error.contains("cannot read"), "{error}");
        assert!(error.contains(&expected_directory.display().to_string()), "{error}");
    }

    #[test]
    fn Test_Snapshot_Directory_Should_Join_Tests_Contract_Surface_Under_The_Given_Root()
    {
        let root = Path::new("/repo");

        assert_eq!(Snapshot_Directory(root), root.join("tests").join("contract").join("surface"));
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
