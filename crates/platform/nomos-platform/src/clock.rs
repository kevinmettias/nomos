//! Time, as a dependency rather than as an ambient fact.

use nomos_contracts::Strategy;

mod timestamp;

pub use timestamp::Timestamp;
pub use timestamp::serialization as timestamp_serde;

/// The source of the current time.
///
/// A dependency rather than a call to [`std::time::SystemTime::now`], for two reasons
/// that both bite. Lease expiry and stale-lock detection are decided by comparing
/// timestamps, and a test that cannot move time forward has to sleep — which makes the
/// suite slow and the failure intermittent. And wall-clock reads are a nondeterminism
/// source: anything on the analysis path that reads a clock directly cannot honestly
/// declare a reproducibility claim, and routing every read through this trait is what
/// makes that checkable rather than aspirational.
/// # What an implementor promises
///
/// The supertrait is [`nomos_contracts::Strategy`], so every implementor states its
/// determinism triple. This is the seam where that question is sharpest and where it had
/// no answer: the twenty-nine types in this workspace that declared a triple were rules,
/// providers and formats, and not one of them was a port -- while the implementations
/// that actually cross the machine boundary, and the doubles that stand in for them,
/// declared nothing. The real one promises nothing and says so; a double built from fixed
/// data reproduces and says that. A caller reading `S::STRENGTH` can tell them apart
/// without knowing either type.
pub trait Clock: Strategy
{
    /// The current time.
    fn Now(&self) -> Timestamp;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::time::Duration;

    /// The instant the elapsed-time cases measure from. An arbitrary real one.
    const AN_INSTANT: i64 = 1_000;

    /// A later instant than `AN_INSTANT`, for the case where the clock reports
    /// the later time first.
    const A_LATER_INSTANT: i64 = 2_000;

    /// How far forward the measured case moves. One constant stands at both the
    /// fixture and the expectation, so the two cannot drift apart.
    const A_FORWARD_STEP: u64 = 30;

    /// An advance from the largest representable instant, which must saturate
    /// rather than wrap.
    const A_SATURATING_ADVANCE: u64 = 60;

    /// An arbitrary real epoch second, far from the zero the type also admits.
    const A_ROUND_TRIP_INSTANT: i64 = 1_700_000_000;

    #[test]
    fn Test_Since_Should_Measure_Forward_Distance()
    {
        let start = Timestamp::From_Unix_Seconds(AN_INSTANT);
        let later = start.Plus(Duration::from_secs(A_FORWARD_STEP));

        assert_eq!(later.Since(start), Duration::from_secs(A_FORWARD_STEP));
    }

    /// A clock that went backwards — a correction, a VM resume, a machine with a bad
    /// battery — must not produce a wildly large elapsed time through underflow. That
    /// would read as "this lease expired an eternity ago" and break every live claim at
    /// once.
    #[test]
    fn Test_Since_Should_Report_Zero_When_The_Clock_Moved_Backwards()
    {
        let earlier = Timestamp::From_Unix_Seconds(AN_INSTANT);
        let later = Timestamp::From_Unix_Seconds(A_LATER_INSTANT);

        assert_eq!(earlier.Since(later), Duration::ZERO);
    }

    #[test]
    fn Test_Plus_Should_Saturate_Rather_Than_Wrap()
    {
        let far_future = Timestamp::From_Unix_Seconds(i64::MAX);

        assert_eq!(
            far_future.Plus(Duration::from_secs(A_SATURATING_ADVANCE)).Unix_Seconds(),
            i64::MAX
        );
    }

    #[test]
    fn Test_From_Unix_Seconds_Should_Round_Trip_Through_Unix_Seconds()
    {
        assert_eq!(
            Timestamp::From_Unix_Seconds(A_ROUND_TRIP_INSTANT).Unix_Seconds(),
            A_ROUND_TRIP_INSTANT
        );
    }
}
