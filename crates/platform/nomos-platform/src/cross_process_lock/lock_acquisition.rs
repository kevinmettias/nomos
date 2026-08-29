//! A held lock, and what taking it displaced.

use crate::StaleTakeover;

/// A held lock, plus whatever had to be broken to get it.
#[derive(Debug)]
pub struct LockAcquisition<Guard>
{
    /// The guard. Dropping it releases the lock.
    pub guard: Guard,
    /// Present when a stale lock was broken. Callers must surface this.
    pub broke_stale: Option<StaleTakeover>,
}
