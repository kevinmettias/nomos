//! A held lock, and what taking it displaced.

use crate::stale_takeover::StaleTakeover;

/// A held lock, plus whatever had to be broken to get it.
#[derive(Debug)]
pub struct LockAcquisition<G>
{
    /// The guard. Dropping it releases the lock.
    pub guard: G,
    /// Present when a stale lock was broken. Callers must surface this.
    pub broke_stale: Option<StaleTakeover>,
}
