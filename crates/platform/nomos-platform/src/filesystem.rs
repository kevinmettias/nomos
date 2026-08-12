//! Reading and writing files, with the durability rules stated rather than assumed.

mod error;

pub use error::FileSystemError;

use std::path::Path;

/// File access.
///
/// Small on purpose. This is not an abstraction over filesystems in general — it is the
/// set of operations Nomos's durable state actually needs, which is read, atomically
/// replace, and check existence.
pub trait FileSystem
{
    /// Reads a file's entire contents as text.
    ///
    /// # Errors
    ///
    /// Returns [`FileSystemError::NotFound`] when the file is absent, and a
    /// [`FileSystemError::Denied`] or [`FileSystemError::Other`] otherwise.
    fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>;

    /// Replaces a file's contents atomically.
    ///
    /// **Write to a temporary file and rename**, never truncate and rewrite. A
    /// truncating write has a window in which the file is empty or half-written, and a
    /// reader that arrives in that window — or a crash that lands in it — sees a
    /// ledger with no items in it. A rename has no such window: a reader sees either
    /// the old contents or the new ones.
    ///
    /// # Errors
    ///
    /// Returns a [`FileSystemError`] when the file could not be replaced. On failure
    /// the previous contents must remain intact.
    fn Replace_Atomically(&self, path: &Path, contents: &str) -> Result<(), FileSystemError>;

    /// Whether a path exists.
    fn Exists(&self, path: &Path) -> bool;
}
