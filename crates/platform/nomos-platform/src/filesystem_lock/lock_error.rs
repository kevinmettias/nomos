//! Every way acquiring a lock fails.

use std::time::Duration;

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

impl std::error::Error for LockError
{}
