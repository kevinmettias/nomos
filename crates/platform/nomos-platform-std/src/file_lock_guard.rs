//! What holding a file lock leaves behind when it is dropped.

use std::path::PathBuf;

/// A lock held on a file. Releasing happens on drop.
#[derive(Debug)]
pub struct FileLockGuard
{
    path: PathBuf,
}

impl FileLockGuard
{
    /// Takes ownership of a lock file, which is released when this value is dropped.
    ///
    /// Deliberately not public. A guard is evidence that its holder took the lock, and a
    /// caller outside this crate that could construct one over any path would be able to
    /// release a lock it never acquired. `FileLock` is the only thing entitled to say a
    /// lock is held, so it is the only thing that can build one of these.
    #[must_use]
    pub(crate) const fn Over(path: PathBuf) -> Self
    {
        return Self { path };
    }
}

impl Drop for FileLockGuard
{
    fn drop(&mut self)
    {
        // A failed release is not worth panicking over — the lock will be broken as
        // stale by whoever comes next, which is exactly the mechanism that exists for
        // holders that go away without cleaning up. Panicking here during unwinding
        // would abort the process and lose the very error being handled.
        let _ = std::fs::remove_file(&self.path);
    }
}
