//! Why a scheduled correction run stopped before its last wave.

/// The one reason a wave-scheduled run stopped early, naming the wave it stopped at.
///
/// A run with no halt reached the end of the partition. A run with one did not, and every
/// wave after the named one was never staged -- which is the point:
/// [`crate::CorrectionSchedule::Scheduled`] says how many waves the substrate computed
/// and [`crate::CorrectionSchedule::Waves`] says how many were attempted, so the gap is
/// exactly what a halt cost.
///
/// What was already committed is *not* rolled back, and
/// [`crate::CorrectionSchedule::Committed`] names it. Waves are independent by
/// construction, so a later wave's refusal is evidence about that wave's own plan against
/// the live workspace and says nothing about whether an earlier wave was right; undoing
/// correct committed work because an unrelated plan went stale would destroy a result
/// nothing found fault with. And the undo the substrate offers,
/// `nomos_corrections::CommittedPlan::Rollback`, refuses once the workspace has moved
/// since that commit, so a cascade of them could refuse partway and leave a tree half
/// undone -- which is not an atomic rollback however it is presented. Reporting what was
/// committed, with each plan's own base and after snapshots, hands the caller exactly
/// what a deliberate rollback needs and keeps the decision where `COR-005`'s "commit or
/// roll back" can actually be made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScheduleHalt
{
    /// A plan in `wave` did not stage, validate, commit or write. `position` is its input
    /// position and `reason` is the refusal, which is also in that wave's own outcomes.
    Refused
    {
        wave: usize,
        position: usize,
        reason: String,
    },
    /// The rerun after `wave` found the affected scope in a state this run had already
    /// been in at `round` -- two corrections undoing each other, or one that changed
    /// nothing. Continuing would carry the scope onward from a state it has already been
    /// carried onward from.
    Repeated
    {
        wave: usize,
        round: usize,
    },
}

impl ScheduleHalt
{
    /// The wave this run stopped at. No wave after it was staged.
    #[must_use]
    pub const fn Wave(&self) -> usize
    {
        return match self
        {
            Self::Refused { wave, .. } | Self::Repeated { wave, .. } => *wave,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const SECOND_WAVE: usize = 1;
    const THIRD_POSITION: usize = 2;
    const FIRST_ROUND: usize = 0;

    #[test]
    fn Test_Wave_Should_Report_The_Wave_A_Refusal_Stopped_At()
    {
        let halt = ScheduleHalt::Refused {
            wave: SECOND_WAVE,
            position: THIRD_POSITION,
            reason: "stale".to_owned(),
        };

        assert_eq!(halt.Wave(), SECOND_WAVE);
    }

    #[test]
    fn Test_Wave_Should_Report_The_Wave_A_Repeated_State_Stopped_At()
    {
        let halt = ScheduleHalt::Repeated {
            wave: SECOND_WAVE,
            round: FIRST_ROUND,
        };

        assert_eq!(halt.Wave(), SECOND_WAVE);
    }
}
