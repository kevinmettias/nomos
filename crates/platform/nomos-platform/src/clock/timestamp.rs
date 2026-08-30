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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The direct address for the constructor -- `clock.rs`'s own suite exercises it
    /// extensively as scaffolding for `Plus`/`Since`, but never names it, which is a
    /// missing address rather than a missing test.
    #[test]
    fn Test_From_Unix_Seconds_Should_Wrap_The_Given_Value()
    {
        assert_eq!(Timestamp::From_Unix_Seconds(1_700_000_000).Unix_Seconds(), 1_700_000_000);
    }

    #[test]
    fn Test_Unix_Seconds_Should_Return_The_Wrapped_Value()
    {
        assert_eq!(Timestamp::From_Unix_Seconds(42).Unix_Seconds(), 42);
    }

    #[test]
    fn Test_Plus_Should_Advance_By_A_Duration()
    {
        let start = Timestamp::From_Unix_Seconds(1_000);

        assert_eq!(start.Plus(std::time::Duration::from_secs(30)).Unix_Seconds(), 1_030);
    }

    #[test]
    fn Test_Since_Should_Measure_The_Gap_Between_Two_Values()
    {
        let earlier = Timestamp::From_Unix_Seconds(1_000);
        let later = Timestamp::From_Unix_Seconds(1_030);

        assert_eq!(later.Since(earlier), std::time::Duration::from_secs(30));
    }
}
