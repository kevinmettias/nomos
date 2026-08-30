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
    /// Releases the lock, and says so on standard error when it cannot.
    ///
    /// # Why the failure is printed rather than returned, panicked or dropped
    ///
    /// A drop has no return value, so there is no caller to hand this to. Panicking is
    /// worse than the failure: a panic during unwinding aborts the process and takes the
    /// original error with it, which is what `check-drop-panics` exists to stop.
    ///
    /// That leaves saying it or losing it, and losing it is not free. The lock file will be
    /// broken as stale by whoever comes next — the mechanism that exists for holders that
    /// die without cleaning up — but only after a full staleness window in which everybody
    /// else is refused with "held by" and this process's name. Silence turns one failed
    /// `remove_file` into somebody else's unexplained wait, minutes later, with nothing
    /// anywhere connecting the two. One line naming the path is the cheapest thing that
    /// closes that gap.
    ///
    /// A path that is already gone is not a failure: the lock is released either way, and a
    /// takeover that judged this holder stale is a normal way for that to happen.
    fn drop(&mut self)
    {
        if let Err(cause) = std::fs::remove_file(&self.path)
            && cause.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!(
                "the lock at {} could not be released and will be held until it goes stale: {cause}",
                self.path.display()
            );
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// `Over` is `pub(crate)`, so this is the only place a test can construct one
    /// directly rather than through `FileLock::Acquire`.
    #[test]
    fn Test_Over_Should_Release_The_File_It_Was_Given_When_Dropped()
    {
        let path = std::env::temp_dir().join(format!("nomos-file-lock-guard-test-{}", std::process::id()));
        std::fs::write(&path, b"held").expect("creates the file the guard will own");

        let guard = FileLockGuard::Over(path.clone());
        assert!(path.exists(), "the guard's path must be the file that was just created");

        drop(guard);

        assert!(!path.exists(), "dropping the guard must release the file it was given");
    }
}
