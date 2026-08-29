//! A cross-process lock built from the one filesystem primitive that is atomic
//! everywhere.

use crate::FileLockGuard;

use nomos_platform::{CrossProcessLock, LockAcquisition, LockError, StaleTakeover};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How long to sleep between attempts while waiting for a lock.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Mutual exclusion via exclusive file creation.
///
/// # Why `create_new` and not an advisory lock
///
/// `OpenOptions::create_new` compiles to `O_CREAT | O_EXCL` on Unix and
/// `CREATE_NEW` on Windows, and it is the only filesystem primitive that is atomic on
/// every platform Nomos targets. `flock` and byte-range locks are not available with
/// the same semantics on Windows, and an implementation that used them would work on
/// the developer's machine and quietly fail to exclude anything on somebody else's.
///
/// # Why staleness is read from metadata
///
/// A holder that died mid-write leaves a truncated or empty lock file. Deciding whether
/// the lock may be broken by parsing its contents would make the decision depend on how
/// far the dead process got — so the age comes from the file's modification time, and
/// the contents are only ever used to *name* the previous holder in the report.
///
/// This type takes no [`nomos_platform::Clock`], deliberately. It would only be used to
/// stamp a diagnostic, and carrying an injectable clock beside a metadata-derived age
/// invites exactly the mistake the age documentation warns about.
pub struct FileLock
{
    path: PathBuf,
}

impl FileLock
{
    /// A lock at the given path.
    ///
    /// The path should sit beside the resource it protects, not inside it — a lock file
    /// within a directory being enumerated shows up as a member of it.
    #[must_use]
    pub fn At(path: impl Into<PathBuf>) -> Self
    {
        return Self { path: path.into() };
    }

    /// The lock file's path.
    #[must_use]
    pub fn Path(&self) -> &Path
    {
        return &self.path;
    }

    /// Attempts to create the lock file exclusively.
    fn Try_Create(&self, holder: &str) -> Result<bool, LockError>
    {
        use std::fs::OpenOptions;

        return match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.path)
        {
            Ok(mut file) => self.Stamped(&mut file, holder),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(error) => Err(LockError::Unusable {
                cause: format!("{}: {error}", self.path.display()),
            }),
        };
    }

    /// Writes the holder line into a lock file this call has just created exclusively.
    ///
    /// # Why a failed stamp refuses the lock rather than degrading quietly
    ///
    /// The lock is the file's existence and not its contents, so an empty lock file still
    /// excludes correctly and this could return `Ok(true)` and say nothing. It does not,
    /// for two reasons. A filesystem that refuses twenty bytes to a file it created a
    /// microsecond ago will refuse the resource the lock protects as well, and finding
    /// that out here — with the write's own words — beats finding it out two steps later
    /// from a replace that cannot say why the disk stopped answering. And an unstamped
    /// lock is the one a takeover report cannot name, so the failure would resurface as
    /// "an unnamed holder" in somebody else's refusal, where nothing connects it back.
    ///
    /// The file is removed before refusing. Leaving it would exclude every holder for a
    /// full staleness window over a failure that may have lasted a moment, which is a
    /// worse outcome than the one this is avoiding. A removal that fails too is carried in
    /// the same sentence rather than replacing it: the caller then knows both that the
    /// lock was not taken and that a file is standing where the lock goes.
    fn Stamped(&self, file: &mut std::fs::File, holder: &str) -> Result<bool, LockError>
    {
        let stamped = format!("{holder}\npid {}\n", std::process::id());
        let Err(cause) = file.write_all(stamped.as_bytes())
        else
        {
            return Ok(true);
        };

        let stranded = match std::fs::remove_file(&self.path)
        {
            Ok(()) => String::new(),
            Err(removal) => format!(", and it could not be removed either: {removal}"),
        };

        return Err(LockError::Unusable {
            cause: format!(
                "{}: the lock file was created but its holder line could not be written: {cause}{stranded}",
                self.path.display()
            ),
        });
    }

    /// How long the lock file has existed, judged from its modification time.
    ///
    /// Compared against the same wall clock the filesystem stamped it with, never
    /// against the injected [`Clock`]. Those are two unrelated time bases, and
    /// subtracting one from the other produces a lock that is never stale on one
    /// machine and always stale on another.
    fn Age(&self) -> Option<Duration>
    {
        return std::fs::metadata(&self.path).ok()?.modified().ok()?.elapsed().ok();
    }

    /// Reads the current holder's name, if the lock file names one.
    fn Current_Holder(&self) -> String
    {
        return std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|contents| contents.lines().next().map(str::to_owned))
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "an unnamed holder".to_owned());
    }

}

impl FileLock
{
    /// Sleeps until the next attempt, or refuses once the caller's patience has run out.
    ///
    /// The refusal names the current holder rather than the one seen when the wait began,
    /// because the useful answer is who has it now — a caller told about a holder that has
    /// since released it would retry against a name that no longer means anything.
    fn Wait_Or_Refuse(
        &self,
        started: std::time::Instant,
        wait_limit: Duration,
    ) -> Result<(), LockError>
    {
        if started.elapsed() >= wait_limit
        {
            return Err(LockError::Held {
                holder: self.Current_Holder(),
                waited: started.elapsed(),
            });
        }
        std::thread::sleep(POLL_INTERVAL);

        return Ok(());
    }

    /// How old the lock file is, if it is old enough to be nobody's.
    ///
    /// A lock file with no readable age is not stale: it may have been created in the
    /// moment between this process finding it and asking about it, and treating an
    /// unanswerable age as stale would break a lock somebody had just taken.
    fn Stale_By(&self, stale_after: Duration) -> Option<Duration>
    {
        return self.Age().filter(|age| return *age >= stale_after);
    }

    /// The lock, now held, and whose stale one had to be removed to get it.
    ///
    /// The takeover travels with the acquisition rather than being logged here: a caller
    /// that broke somebody else's lock is entitled to say so in its own words, and one that
    /// did not must not have to check.
    fn Held(&self, broke_stale: Option<StaleTakeover>) -> LockAcquisition<FileLockGuard>
    {
        return LockAcquisition {
            guard: FileLockGuard::Over(self.path.clone()),
            broke_stale,
        };
    }

    /// Removes a lock file old enough to be nobody's, and says whose it was.
    ///
    /// Remove and retry rather than assuming the removal won us the lock. Two processes can
    /// decide the same lock is stale at the same moment; only the one whose subsequent
    /// `create_new` succeeds actually holds it, and going back through the loop is what
    /// makes that true rather than assumed.
    fn Break_Stale(&self, age: Duration) -> Option<StaleTakeover>
    {
        let previous_holder = self.Current_Holder();
        if std::fs::remove_file(&self.path).is_err()
        {
            return None;
        }

        return Some(StaleTakeover {
            previous_holder,
            age,
        });
    }
}

impl CrossProcessLock for FileLock
{
    type Guard = FileLockGuard;

    fn Acquire(
        &self,
        holder: &str,
        wait_limit: Duration,
        stale_after: Duration,
    ) -> Result<LockAcquisition<Self::Guard>, LockError>
    {
        let started = std::time::Instant::now();
        let mut broke_stale = None;

        let acquired = loop
        {
            if self.Try_Create(holder)?
            {
                break self.Held(broke_stale);
            }
            if let Some(age) = self.Stale_By(stale_after)
            {
                broke_stale = self.Break_Stale(age).or(broke_stale);
                continue;
            }

            self.Wait_Or_Refuse(started, wait_limit)?;
        };

        return Ok(acquired);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Temporary_Path(name: &str) -> PathBuf
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-lock-test-{name}-{}", std::process::id()));

        // A path that is already absent is the state this asks for, so `NotFound` is
        // success. Anything else is said out loud rather than discarded, because a setup
        // that quietly cannot delete hands the test a lock file a previous run left behind
        // — and every one of these tests reads "the lock file exists" as "somebody holds it".
        if let Err(cause) = std::fs::remove_file(&path)
            && cause.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!("{} could not be cleared: {cause}", path.display());
        }

        return path;
    }

    const NO_WAIT: Duration = Duration::ZERO;
    const NEVER_STALE: Duration = Duration::from_secs(86_400);
    /// Makes any existing lock immediately eligible for takeover, which is how these
    /// tests reach the stale path without sleeping. The production threshold is minutes.
    const ALWAYS_STALE: Duration = Duration::ZERO;

    #[test]
    fn Test_An_Uncontended_Lock_Should_Be_Acquired_Cleanly()
    {
        let path = Temporary_Path("uncontended");
        let lock = FileLock::At(&path);

        let acquisition = lock.Acquire("agent-a", NO_WAIT, NEVER_STALE).unwrap();

        assert!(acquisition.broke_stale.is_none());
        assert!(path.exists());
        drop(acquisition);
        assert!(!path.exists(), "dropping the guard must release the lock");
    }

    /// The property the whole type exists for. Two holders must not both believe they
    /// won.
    #[test]
    fn Test_A_Held_Lock_Should_Refuse_A_Second_Holder()
    {
        let path = Temporary_Path("contended");
        let lock = FileLock::At(&path);

        let _held = lock.Acquire("agent-a", NO_WAIT, NEVER_STALE).unwrap();
        let refused = lock.Acquire("agent-b", NO_WAIT, NEVER_STALE);

        assert!(matches!(refused, Err(LockError::Held { .. })));
    }

    /// A refusal must name who is holding it, or the user's only recourse is to guess.
    #[test]
    fn Test_A_Refusal_Should_Name_The_Current_Holder()
    {
        let path = Temporary_Path("named-holder");
        let lock = FileLock::At(&path);

        let _held = lock.Acquire("agent-a", NO_WAIT, NEVER_STALE).unwrap();

        match lock.Acquire("agent-b", NO_WAIT, NEVER_STALE)
        {
            Err(LockError::Held { holder, .. }) => assert_eq!(holder, "agent-a"),
            // A test assertion, not production control flow: any other outcome here is this
            // test's own failure mode, and panicking is how a test reports one.
            other => panic!("expected a Held refusal naming agent-a, got {other:?}"),
        }
    }

    /// A crashed holder must not block every future writer forever. `mem::forget` is
    /// exactly what a killed process leaves behind: a lock file with no live owner and
    /// no destructor coming.
    #[test]
    fn Test_A_Stale_Lock_Should_Be_Broken_And_Reported()
    {
        let path = Temporary_Path("stale");
        let lock = FileLock::At(&path);

        let abandoned = lock.Acquire("agent-a", NO_WAIT, NEVER_STALE).unwrap();
        // Simulates a crashed holder: a killed process never runs the guard's destructor, so
        // forgetting it here (rather than dropping it) is what leaves the lock file behind for
        // the takeover this test is about to exercise.
        std::mem::forget(abandoned);

        let taken_over = lock.Acquire("agent-b", NO_WAIT, ALWAYS_STALE).unwrap();

        let takeover = taken_over
            .broke_stale
            .as_ref()
            .expect("breaking a stale lock must be reported, never silent");
        assert_eq!(
            takeover.previous_holder, "agent-a",
            "the report must name who abandoned the lock, because whether their update              landed is a question only a person can answer"
        );
    }

    /// The negative control for the test above, and the one that makes it mean
    /// something. A lock that is merely held, not stale, must not be broken however
    /// impatient the caller is — otherwise the staleness threshold is decoration and
    /// the lock excludes nothing.
    #[test]
    fn Test_A_Fresh_Lock_Should_Not_Be_Broken()
    {
        let path = Temporary_Path("fresh");
        let lock = FileLock::At(&path);

        let held = lock.Acquire("agent-a", NO_WAIT, NEVER_STALE).unwrap();
        let refused = lock.Acquire("agent-b", NO_WAIT, NEVER_STALE);

        assert!(
            matches!(refused, Err(LockError::Held { .. })),
            "a lock younger than the staleness threshold must not be broken"
        );
        drop(held);
    }

    /// A clean acquisition must not claim a takeover. If this ever fails, every
    /// acquisition reports a phantom crash, and a report that fires constantly is one
    /// nobody reads — which is the same as not reporting it at all.
    #[test]
    fn Test_A_Clean_Acquisition_Should_Not_Report_A_Takeover()
    {
        let path = Temporary_Path("clean");
        let lock = FileLock::At(&path);

        let first = lock.Acquire("agent-a", NO_WAIT, ALWAYS_STALE).unwrap();
        drop(first);
        let second = lock.Acquire("agent-b", NO_WAIT, ALWAYS_STALE).unwrap();

        assert!(
            second.broke_stale.is_none(),
            "the previous holder released cleanly; there was nothing to break"
        );
    }
}
