//! The temporary trees this crate's tests walk and profile.
//!
//! Shared between the walk's own tests in the crate root and [`crate::WorkspaceProfile`]'s,
//! because both build the same kind of fixture -- a fresh directory under the process's own
//! temporary directory, files written into it, the tree removed again -- and a second copy of
//! three helpers would be the duplication `OD-HOST-008` closed one level up.

use std::path::PathBuf;

/// A fresh, empty directory under the temporary directory, named for the test that owns it.
pub(crate) fn Fresh_Root(name: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");
    return root;
}

/// Writes one fixture file, whose parent directory a [`Make_Fixture_Directory`] call has
/// already created.
///
/// Same promise as [`Make_Fixture_Directory`], from the same [`Fresh_Root`] the caller
/// just made: the path lies under a temporary tree this test's own process owns.
pub(crate) fn Write_Fixture(path: PathBuf, text: &str)
{
    std::fs::write(path, text).expect("the fixture root is a tempdir this test owns");
}

/// Writes one fixture file whose bytes are deliberately not text, for a test that needs a
/// file which is there and cannot be read as a string.
///
/// Same promise as [`Write_Fixture`]: the path lies under a temporary tree this test's own
/// process owns.
pub(crate) fn Write_Fixture_Bytes(path: PathBuf, bytes: &[u8])
{
    std::fs::write(path, bytes).expect("the fixture root is a tempdir this test owns");
}

/// Creates one directory under a fixture root, so a file can then be written inside it.
///
/// The promise holds because every caller passes `root.join(...)` for the [`Fresh_Root`] it
/// made moments earlier in this test's own process.
pub(crate) fn Make_Fixture_Directory(path: PathBuf)
{
    std::fs::create_dir_all(path).expect("the fixture root is a tempdir this test owns");
}
