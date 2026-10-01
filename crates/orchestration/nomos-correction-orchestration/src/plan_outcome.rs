//! What one scheduled plan did in the wave it was placed in.

/// The end of one plan's own trip through `nomos-corrections`' lifecycle, inside the wave
/// the substrate placed it in.
///
/// Per plan rather than per run, which is the whole difference between this and
/// [`crate::CorrectionOutcome`]: a wave-scheduled run carries several plans at once, so
/// "what happened" is a list and not a variant. `path` and `summary` are the same two
/// values `CorrectionOutcome` carries, unchanged -- each family's own
/// `nomos_corrections::CorrectionCandidate::Description`, rendered verbatim by whoever
/// reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanOutcome
{
    /// Staged and validated against the run's workspace, and deliberately not committed:
    /// the run was not asked to commit.
    Staged
    {
        path: String,
        summary: String,
    },
    /// Staged, validated, committed through the workspace's one door, and written to
    /// disk. `base` and `after_snapshot` are the workspace's own snapshots either side of
    /// the commit, which is what a caller would need to undo it deliberately.
    Committed
    {
        path: String,
        summary: String,
        base: String,
        after_snapshot: String,
    },
    /// Refused at stage, validate, commit or write, carrying the refusal that ended it
    /// verbatim. A plan that refuses ends the run: see [`crate::ScheduleHalt`].
    Refused
    {
        path: String,
        reason: String,
    },
}

impl PlanOutcome
{
    /// The path this outcome is about, whichever kind it is.
    #[must_use]
    pub fn Path(&self) -> &str
    {
        return match self
        {
            Self::Staged { path, .. } | Self::Committed { path, .. } | Self::Refused { path, .. } => path,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Path_Should_Report_The_Staged_Plans_Path()
    {
        let outcome = PlanOutcome::Staged {
            path: "a.rs".to_owned(),
            summary: "struck".to_owned(),
        };

        assert_eq!(outcome.Path(), "a.rs");
    }

    #[test]
    fn Test_Path_Should_Report_The_Committed_Plans_Path()
    {
        let outcome = PlanOutcome::Committed {
            path: "b.rs".to_owned(),
            summary: "stripped".to_owned(),
            base: "base".to_owned(),
            after_snapshot: "after".to_owned(),
        };

        assert_eq!(outcome.Path(), "b.rs");
    }

    #[test]
    fn Test_Path_Should_Report_The_Refused_Plans_Path()
    {
        let outcome = PlanOutcome::Refused {
            path: "c.rs".to_owned(),
            reason: "stale".to_owned(),
        };

        assert_eq!(outcome.Path(), "c.rs");
    }
}
