//! What one wave held, what each of its plans did, and what the rerun after it saw.

use crate::PlanOutcome;

/// One wave of a scheduled correction run: which plans the substrate put in it, what each
/// of them did, and how the affected scope moved once the wave was committed.
///
/// `positions` are input positions, exactly as `nomos_corrections::WavePartition` reports
/// them, because a `nomos_corrections::CorrectionPlan` has no identity of its own and a
/// position is what ties a wave back to the plan a caller handed in. Every plan in one
/// wave is independent of every other in it by `nomos_corrections::Compatibility`'s
/// judgment over their declared read and write sets -- that is what a wave *means*, and
/// it is computed rather than assumed.
///
/// `cleared` and `introduced` are `COR-005`'s "compare state signatures" half, read as
/// text rather than as a digest: what the rerun after this wave stopped reporting, and
/// what it started reporting. A correction that reintroduces a finding shows up in
/// `introduced` by name, which is the thing a digest comparison alone cannot say. Both
/// are empty for a wave that was not committed, because a dry run writes nothing and so
/// cannot move the scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaveReport
{
    pub(crate) positions: Vec<usize>,
    pub(crate) outcomes: Vec<PlanOutcome>,
    pub(crate) cleared: Vec<String>,
    pub(crate) introduced: Vec<String>,
}

impl WaveReport
{
    /// A report for a wave holding `positions`, before any of its plans has run.
    pub(crate) fn Of(positions: Vec<usize>) -> Self
    {
        return Self {
            positions,
            outcomes: Vec::new(),
            cleared: Vec::new(),
            introduced: Vec::new(),
        };
    }

    /// The input positions of every plan the substrate placed in this wave, ascending.
    #[must_use]
    pub fn Positions(&self) -> &[usize]
    {
        return &self.positions;
    }

    /// What each plan this run attempted in this wave did, in the order it attempted
    /// them. Shorter than [`Self::Positions`] when one of them refused, because the plans
    /// after a refusal are not attempted.
    #[must_use]
    pub fn Outcomes(&self) -> &[PlanOutcome]
    {
        return &self.outcomes;
    }

    /// What the rerun after this wave stopped reporting over the affected scope.
    #[must_use]
    pub fn Cleared(&self) -> &[String]
    {
        return &self.cleared;
    }

    /// What the rerun after this wave started reporting over the affected scope -- a
    /// finding this wave's own corrections introduced or reintroduced.
    #[must_use]
    pub fn Introduced(&self) -> &[String]
    {
        return &self.introduced;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const FIRST: usize = 0;
    const THIRD: usize = 2;

    #[test]
    fn Test_Of_Should_Carry_The_Positions_And_Nothing_Else()
    {
        let report = WaveReport::Of(vec![FIRST, THIRD]);

        assert_eq!(report.Positions(), [FIRST, THIRD]);
        assert!(report.Outcomes().is_empty());
        assert!(report.Cleared().is_empty());
        assert!(report.Introduced().is_empty());
    }

    #[test]
    fn Test_Outcomes_Should_Report_What_Was_Recorded()
    {
        let mut report = WaveReport::Of(vec![FIRST]);
        report.outcomes.push(PlanOutcome::Staged {
            path: "a.rs".to_owned(),
            summary: "struck".to_owned(),
        });

        assert_eq!(report.Outcomes().len(), 1);
        assert_eq!(report.Outcomes().first().map(PlanOutcome::Path), Some("a.rs"));
    }

    #[test]
    fn Test_Cleared_And_Introduced_Should_Report_What_The_Rerun_Compared()
    {
        let mut report = WaveReport::Of(vec![FIRST]);
        report.cleared = vec!["gone".to_owned()];
        report.introduced = vec!["new".to_owned()];

        assert_eq!(report.Cleared(), ["gone".to_owned()]);
        assert_eq!(report.Introduced(), ["new".to_owned()]);
    }
}
