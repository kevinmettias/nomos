//! What a wave-scheduled correction run planned, ran, refused and observed.

use crate::{PlanOutcome, ScheduleHalt, WaveReport};
use nomos_corrections::UnresolvedAccess;

/// The whole answer [`crate::Run_Correction_Waves`] gives: the plans a set of findings
/// claimed, the waves `nomos_corrections::WavePartition` grouped them into, what each
/// wave did, what was committed, and what the reruns between waves saw.
///
/// One value rather than one variant per ending, which is where this differs from
/// [`crate::CorrectionOutcome`]: a scheduled run can refuse one claim, commit two waves
/// and stop at a third all in the same run, and an enum would have had to pick one of
/// those to be the answer. Nothing here is rendered; a host maps it to text or to a wire
/// exactly as it already does for `CorrectionOutcome`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrectionSchedule
{
    pub(crate) unbuilt: Vec<String>,
    pub(crate) unschedulable: Option<UnresolvedAccess>,
    pub(crate) scheduled: usize,
    pub(crate) waves: Vec<WaveReport>,
    pub(crate) halt: Option<ScheduleHalt>,
    pub(crate) rounds: usize,
}

impl CorrectionSchedule
{
    /// A run whose claims became `scheduled` waves, before any of them has been staged.
    pub(crate) fn Of(unbuilt: Vec<String>, scheduled: usize) -> Self
    {
        return Self {
            unbuilt,
            unschedulable: None,
            scheduled,
            waves: Vec::new(),
            halt: None,
            rounds: 0,
        };
    }

    /// A run the partition refused outright, carrying the substrate's own refusal.
    pub(crate) fn Refusing(unbuilt: Vec<String>, refusal: UnresolvedAccess) -> Self
    {
        let mut schedule = Self::Of(unbuilt, 0);
        schedule.unschedulable = Some(refusal);

        return schedule;
    }

    /// Claims a family recognized but could not build a plan for, each with the refusal
    /// that stopped it. Reported rather than dropped: a claim that silently vanished
    /// would look exactly like a tree that never had it.
    #[must_use]
    pub fn Unbuilt(&self) -> &[String]
    {
        return &self.unbuilt;
    }

    /// `nomos_corrections::WavePartition`'s own refusal, when it declined to place a
    /// plan whose declared read or write set is not known complete.
    ///
    /// The partition refuses whole rather than per plan -- unknown independence is not
    /// safe parallelism, and waves built around a hole would be exactly that -- so this
    /// being `Some` means no wave ran at all. The refusal names the plan's position, the
    /// candidate that carried the set, its tier and what its provider claimed, which is
    /// everything the substrate knows and more than a dropped plan would have said.
    #[must_use]
    pub const fn Unschedulable(&self) -> Option<&UnresolvedAccess>
    {
        return self.unschedulable.as_ref();
    }

    /// How many waves the substrate computed for this run's plans.
    #[must_use]
    pub const fn Scheduled(&self) -> usize
    {
        return self.scheduled;
    }

    /// The waves this run actually attempted, in order. Shorter than [`Self::Scheduled`]
    /// exactly when [`Self::Halt`] stopped it.
    #[must_use]
    pub fn Waves(&self) -> &[WaveReport]
    {
        return &self.waves;
    }

    /// Why this run stopped before its last wave, or `None` when it did not.
    #[must_use]
    pub const fn Halt(&self) -> Option<&ScheduleHalt>
    {
        return self.halt.as_ref();
    }

    /// How many times the affected scope was observed: once before the first wave, and
    /// once after each committed wave. Zero for a run that was not asked to commit, which
    /// writes nothing and so cannot move the scope.
    #[must_use]
    pub const fn Rounds(&self) -> usize
    {
        return self.rounds;
    }

    /// Every path this run committed, in the order it committed them.
    ///
    /// What a caller holds when a later wave halted the run: committed work is kept, and
    /// this is how it is reported rather than silently kept. [`ScheduleHalt`] carries the
    /// argument for keeping it; each path's own `PlanOutcome::Committed` carries the
    /// snapshots a deliberate rollback would need.
    #[must_use]
    pub fn Committed(&self) -> Vec<&str>
    {
        return self.waves.iter().flat_map(|wave| return wave.Outcomes()).filter_map(Committed_Path).collect();
    }
}

/// The path `outcome` committed, or `None` when it committed nothing. Matched
/// exhaustively rather than with a wildcard, so a fourth outcome kind cannot quietly
/// count as "not committed".
fn Committed_Path(outcome: &PlanOutcome) -> Option<&str>
{
    return match outcome
    {
        PlanOutcome::Committed { path, .. } => Some(path.as_str()),
        PlanOutcome::Staged { .. } | PlanOutcome::Refused { .. } => None,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::Unresolved_Refusal;

    const ONE_WAVE: usize = 1;
    const TWO_WAVES: usize = 2;
    const SECOND_WAVE: usize = 1;
    const FIRST: usize = 0;

    fn Committed_Outcome(path: &str) -> PlanOutcome
    {
        return PlanOutcome::Committed {
            path: path.to_owned(),
            summary: "fixed".to_owned(),
            base: "base".to_owned(),
            after_snapshot: "after".to_owned(),
        };
    }

    fn Wave_Holding(outcomes: Vec<PlanOutcome>) -> WaveReport
    {
        let mut report = WaveReport::Of(vec![FIRST]);
        report.outcomes = outcomes;

        return report;
    }

    #[test]
    fn Test_Of_Should_Start_With_Nothing_Run()
    {
        let schedule = CorrectionSchedule::Of(Vec::new(), ONE_WAVE);

        assert_eq!(schedule.Scheduled(), ONE_WAVE);
        assert!(schedule.Waves().is_empty());
        assert!(schedule.Halt().is_none());
        assert!(schedule.Unschedulable().is_none());
        assert_eq!(schedule.Rounds(), 0);
    }

    /// A refused partition schedules nothing, and the refusal it carries is the
    /// substrate's own value rather than a sentence rendered from it.
    #[test]
    fn Test_Refusing_Should_Carry_The_Substrates_Own_Refusal_And_Schedule_Nothing()
    {
        let refusal = Unresolved_Refusal();

        let schedule = CorrectionSchedule::Refusing(Vec::new(), refusal.clone());

        assert_eq!(schedule.Unschedulable(), Some(&refusal));
        assert_eq!(schedule.Scheduled(), 0);
        assert!(schedule.Waves().is_empty());
    }

    #[test]
    fn Test_Unbuilt_Should_Report_Every_Claim_That_Could_Not_Become_A_Plan()
    {
        let schedule = CorrectionSchedule::Of(vec!["`a.rs`: ambiguous".to_owned()], 0);

        assert_eq!(schedule.Unbuilt(), ["`a.rs`: ambiguous".to_owned()]);
    }

    #[test]
    fn Test_Committed_Should_Name_Every_Committed_Path_In_Order()
    {
        let mut schedule = CorrectionSchedule::Of(Vec::new(), TWO_WAVES);
        schedule.waves.push(Wave_Holding(vec![Committed_Outcome("a.rs"), Committed_Outcome("b.rs")]));
        schedule.waves.push(Wave_Holding(vec![PlanOutcome::Refused {
            path: "c.rs".to_owned(),
            reason: "stale".to_owned(),
        }]));

        assert_eq!(schedule.Committed(), ["a.rs", "b.rs"]);
    }

    #[test]
    fn Test_Committed_Should_Not_Count_A_Staged_Plan()
    {
        let mut schedule = CorrectionSchedule::Of(Vec::new(), ONE_WAVE);
        schedule.waves.push(Wave_Holding(vec![PlanOutcome::Staged {
            path: "a.rs".to_owned(),
            summary: "struck".to_owned(),
        }]));

        assert!(schedule.Committed().is_empty());
    }

    #[test]
    fn Test_Halt_Should_Report_What_Stopped_The_Run()
    {
        let mut schedule = CorrectionSchedule::Of(Vec::new(), TWO_WAVES);
        schedule.halt = Some(ScheduleHalt::Repeated {
            wave: SECOND_WAVE,
            round: FIRST,
        });

        assert_eq!(schedule.Halt().map(ScheduleHalt::Wave), Some(SECOND_WAVE));
    }
}
