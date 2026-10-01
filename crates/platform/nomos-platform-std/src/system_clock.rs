//! The system clock.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform::{Clock, Timestamp};

/// Reads the operating system's wall clock.
///
/// The only implementation that should exist outside tests. Every other component takes
/// a [`Clock`] rather than calling [`std::time::SystemTime::now`] directly, which is
/// what lets a test move time forward instead of sleeping — and what keeps a wall-clock
/// read from appearing on a path that has declared a reproducibility guarantee.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

/// Reaches the real machine, so it reproduces nothing and says so.
impl Strategy for SystemClock
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

impl Clock for SystemClock
{
    fn Now(&self) -> Timestamp
    {
        let since_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        return Timestamp::From_Unix_Seconds(i64::try_from(since_epoch.as_secs()).unwrap_or(0));
    }
}
