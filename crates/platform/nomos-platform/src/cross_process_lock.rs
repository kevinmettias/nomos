//! Mutual exclusion between processes that cannot see each other.

mod lock_acquisition;
mod lock_error;
mod stale_takeover;

pub use lock_acquisition::LockAcquisition;
pub use lock_error::LockError;
pub use stale_takeover::StaleTakeover;

use std::time::Duration;

/// Exclusion across processes that share a filesystem and nothing else.
///
/// The contract has three parts, and each of them was a defect somewhere before it was
/// a rule:
///
/// 1. **Acquisition is atomic.** Two processes racing must not both believe they won.
/// 2. **A stale lock can be broken, and breaking one is reported.** Otherwise a crashed
///    holder blocks every future writer forever, and the only remedy is a human
///    deleting a file they have to know exists.
/// 3. **Staleness is judged from filesystem metadata, never from the lock file's
///    contents.** A holder that died mid-write leaves a truncated file; parsing it to
///    decide whether the lock may be broken makes the decision depend on how far the
///    dead process got. The contents are used only to *name* the previous holder in the
///    report, where being unable to read them costs a diagnostic rather than a decision.
///
/// Note that staleness is deliberately not measured against an injectable clock. The
/// lock file's age comes from the filesystem, so the only self-consistent comparison is
/// against the same wall clock the filesystem stamped it with. An injected clock here
/// would put two unrelated time bases on either side of one subtraction — which is a
/// bug that presents as a lock that is never stale, or as one that is always stale, and
/// which does so only on the machine where the two happen to disagree.
pub trait CrossProcessLock
{
    /// The guard type released on drop.
    type Guard;

    /// Takes the lock, waiting up to `wait_limit`, breaking it if it is older than
    /// `stale_after`.
    ///
    /// # Errors
    ///
    /// Returns [`LockError::Held`] if a live holder keeps it for the whole wait, and
    /// [`LockError::Unusable`] if the lock cannot be manipulated at all.
    fn Acquire(
        &self,
        holder: &str,
        wait_limit: Duration,
        stale_after: Duration,
    ) -> Result<LockAcquisition<Self::Guard>, LockError>;
}
