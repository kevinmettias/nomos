//! A point in time, as everything in Nomos records one.

use serde::{Deserialize, Serialize};

/// A point in time, as whole seconds since the Unix epoch.
///
/// Second resolution is deliberate. These timestamps appear in the work ledger, which
/// is committed and read under `git diff`; sub-second precision would make every write
/// churn without telling a reader anything they act on. Where ordering matters more
/// finely than a second, the ordering is carried explicitly rather than inferred from a
/// clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp
{
    /// Wraps whole seconds since the Unix epoch.
    #[must_use]
    pub const fn From_Unix_Seconds(seconds: i64) -> Self
    {
        return Self(seconds);
    }

    /// Whole seconds since the Unix epoch.
    #[must_use]
    pub const fn Unix_Seconds(self) -> i64
    {
        return self.0;
    }

    /// This timestamp advanced by a duration, saturating at the representable range.
    #[must_use]
    pub fn Plus(self, duration: std::time::Duration) -> Self
    {
        let seconds = i64::try_from(duration.as_secs()).unwrap_or(i64::MAX);
        return Self(self.0.saturating_add(seconds));
    }

    /// How long after `earlier` this timestamp is, or zero if it is not after it.
    #[must_use]
    pub fn Since(self, earlier: Self) -> std::time::Duration
    {
        let elapsed = self.0.saturating_sub(earlier.0);
        return std::time::Duration::from_secs(u64::try_from(elapsed).unwrap_or(0));
    }
}
