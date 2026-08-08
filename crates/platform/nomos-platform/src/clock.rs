//! Time, as a dependency rather than as an ambient fact.

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

/// The source of the current time.
///
/// A dependency rather than a call to [`std::time::SystemTime::now`], for two reasons
/// that both bite. Lease expiry and stale-lock detection are decided by comparing
/// timestamps, and a test that cannot move time forward has to sleep — which makes the
/// suite slow and the failure intermittent. And wall-clock reads are a nondeterminism
/// source: anything on the analysis path that reads a clock directly cannot honestly
/// declare a reproducibility claim, and routing every read through this trait is what
/// makes that checkable rather than aspirational.
pub trait Clock
{
    /// The current time.
    fn Now(&self) -> Timestamp;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::time::Duration;

    #[test]
    fn Test_Elapsed_Should_Measure_Forward_Distance()
    {
        let start = Timestamp::From_Unix_Seconds(1_000);
        let later = start.Plus(Duration::from_secs(30));

        assert_eq!(later.Since(start), Duration::from_secs(30));
    }

    /// A clock that went backwards — a correction, a VM resume, a machine with a bad
    /// battery — must not produce a wildly large elapsed time through underflow. That
    /// would read as "this lease expired an eternity ago" and break every live claim at
    /// once.
    #[test]
    fn Test_A_Backwards_Clock_Should_Report_No_Elapsed_Time()
    {
        let earlier = Timestamp::From_Unix_Seconds(1_000);
        let later = Timestamp::From_Unix_Seconds(2_000);

        assert_eq!(earlier.Since(later), Duration::ZERO);
    }

    #[test]
    fn Test_Advancing_Should_Saturate_Rather_Than_Wrap()
    {
        let far_future = Timestamp::From_Unix_Seconds(i64::MAX);

        assert_eq!(
            far_future.Plus(Duration::from_secs(60)).Unix_Seconds(),
            i64::MAX
        );
    }
}
