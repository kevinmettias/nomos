//! Time, as a dependency rather than as an ambient fact.

use crate::timestamp::Timestamp;


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
