//! What was found when a lock was taken from a holder that had stopped.

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
