//! What a step's declared `Timeout` measured over one attempt at its dispatch.

use std::num::NonZeroU32;

/// What a step's declared `nomos_contracts::Timeout` measured over one attempt.
///
/// Four states rather than a bare `bool`, because "the bound was honored" and "nobody
/// measured the bound" are not the same answer and collapsing them is the exact failure
/// this type exists to prevent: [`Self::Unmeasured`] is what a run with no clock reports
/// for a step that declared a bound, so an overrun is never reported as honored by a run
/// that measured nothing at all.
///
/// Seconds, because `nomos_contracts::Timeout::Seconds` is declared in seconds and
/// `nomos_platform::Timestamp` is a second-resolution instant -- this type does not
/// invent a finer resolution than either of the two values it compares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepTiming
{
    /// The step declared `Timeout::Unbounded`, so there was no bound to measure against.
    Unbounded,
    /// The step declared a bound and no clock was supplied, so nothing measured whether
    /// the dispatch stayed inside it.
    Unmeasured
    {
        /// The bound the step declared and this run did not measure.
        declared_seconds: NonZeroU32,
    },
    /// The step declared a bound and the measured dispatch finished inside it.
    Honored
    {
        /// The bound the step declared.
        declared_seconds: NonZeroU32,
        /// How long the dispatch took, as the supplied clock measured it.
        elapsed_seconds: u64,
    },
    /// The step declared a bound and the measured dispatch exceeded it -- timed out, not
    /// merely slow.
    ///
    /// Reported rather than interrupted: this crate composes no cancellation runtime and
    /// `OD-WORKFLOW-005` declines one, so a dispatch already under way is waited on to
    /// its end and the overrun is reported afterward. What the declaration buys is that a
    /// caller is told the bound was broken, not that the step was cut short.
    Exceeded
    {
        /// The bound the step declared.
        declared_seconds: NonZeroU32,
        /// How long the dispatch took, as the supplied clock measured it.
        elapsed_seconds: u64,
    },
}

impl StepTiming
{
    /// Whether this attempt broke the bound its step declared.
    ///
    /// False for [`Self::Unmeasured`], which is not a claim that the bound held -- only
    /// that nothing measured it. A caller that needs the difference reads the variant.
    #[must_use]
    pub const fn Has_Timed_Out(self) -> bool
    {
        return matches!(self, Self::Exceeded { .. });
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The bound these cases declare. An arbitrary real one; the number is not what any
    /// case varies.
    const DECLARED_SECONDS: u32 = 30;

    /// How long a dispatch that broke `DECLARED_SECONDS` took.
    const OVERRUN_SECONDS: u64 = 31;

    fn Declared() -> NonZeroU32
    {
        return NonZeroU32::new(DECLARED_SECONDS).expect("DECLARED_SECONDS is nonzero");
    }

    #[test]
    fn Test_Only_An_Exceeded_Bound_Should_Report_Having_Timed_Out()
    {
        assert!(StepTiming::Exceeded { declared_seconds: Declared(), elapsed_seconds: OVERRUN_SECONDS }.Has_Timed_Out());
        assert!(!StepTiming::Honored { declared_seconds: Declared(), elapsed_seconds: 0 }.Has_Timed_Out());
        assert!(!StepTiming::Unbounded.Has_Timed_Out());
    }

    /// The distinction this type exists for: a declared bound nothing measured must not
    /// answer the timed-out question as though the bound had held.
    #[test]
    fn Test_An_Unmeasured_Bound_Should_Be_Neither_Honored_Nor_Timed_Out()
    {
        let unmeasured = StepTiming::Unmeasured { declared_seconds: Declared() };

        assert!(!unmeasured.Has_Timed_Out());
        assert_ne!(unmeasured, StepTiming::Honored { declared_seconds: Declared(), elapsed_seconds: 0 });
    }
}
