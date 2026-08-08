//! Mutual exclusion between processes that cannot see each other.

use std::time::Duration;

/// A previous lock holder that went away without releasing.
///
/// The takeover is **reported**, never silent. The previous holder abandoned an update
/// partway through, and whether that update landed is a question only a person can
/// settle — the process that broke the lock has no way to know, and a system that
/// quietly proceeded would be answering it by assumption.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaleTakeover
{
    /// Who held the lock, as far as the lock file recorded it.
    pub previous_holder: String,
    /// How long the lock had existed when it was broken.
    pub age: Duration,
}

/// A held lock, plus whatever had to be broken to get it.
#[derive(Debug)]
pub struct LockAcquisition<G>
{
    /// The guard. Dropping it releases the lock.
    pub guard: G,
    /// Present when a stale lock was broken. Callers must surface this.
    pub broke_stale: Option<StaleTakeover>,
}

/// Why a lock could not be taken.
#[derive(Debug)]
pub enum LockError
{
    /// Another live holder has it, and it has not gone stale.
    Held
    {
        /// Who holds it.
        holder: String,
        /// How long the caller waited before giving up.
        waited: Duration,
    },
    /// The lock file could not be created, read or removed.
    Unusable
    {
        /// What went wrong.
        cause: String,
    },
}

impl core::fmt::Display for LockError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Held { holder, waited } => write!(
                formatter,
                "lock is held by {holder}; waited {waited:?} without it being released"
            ),
            Self::Unusable { cause } => write!(formatter, "lock is unusable: {cause}"),
        };
    }
}

impl std::error::Error for LockError {}

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
