//! A plan that has been submitted through the door, and the reverse that undoes it.

use crate::staged::Assert_Not_Moved;
use crate::CorrectionError;
use nomos_contracts::SnapshotId;
use nomos_workspace::{Workspace, WorkspaceChangeSet};

/// A plan whose forward change has been applied to a live workspace.
///
/// Carries the exact reverse of what it applied, built from the same edits at staging
/// time — not recomputed from whatever the workspace holds when rollback is asked for, so
/// rollback undoes what this plan did rather than whatever the paths currently say.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommittedPlan
{
    base: SnapshotId,
    after: SnapshotId,
    reverse: WorkspaceChangeSet,
}

impl CommittedPlan
{
    pub(crate) fn Of(base: SnapshotId, after: SnapshotId, reverse: WorkspaceChangeSet) -> Self
    {
        return Self {
            base,
            after,
            reverse,
        };
    }

    #[must_use]
    pub const fn Base(&self) -> SnapshotId
    {
        return self.base;
    }

    #[must_use]
    pub const fn After(&self) -> SnapshotId
    {
        return self.after;
    }

    /// Submits this plan's reverse change through `live`'s one door, undoing it.
    ///
    /// # Errors
    ///
    /// Returns [`CorrectionError::Moved`] if `live` has moved since this plan was
    /// committed, or [`CorrectionError::Workspace`] if the workspace door itself refuses
    /// the reverse change.
    pub fn Rollback(self, live: &mut Workspace) -> Result<SnapshotId, CorrectionError>
    {
        Assert_Not_Moved(self.after, live)?;

        live.Apply(&self.reverse)?;

        return Ok(live.Id());
    }
}

#[cfg(test)]
mod tests
{
    use crate::{ChangeSet, CommittedPlan, CorrectionCandidate, CorrectionPlan, Edit};
    use nomos_contracts::{ConfigurationId, Digest128};
    use nomos_model::Content_Digest;
    use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};

    fn Base() -> Workspace
    {
        let variant = BuildVariant::New("x86_64-unknown-none", "test", "fixed", Vec::<String>::new());
        let configuration = ConfigurationId::From_Digest(Digest128::From_Bytes([0x33; 16]));
        let mut workspace = Workspace::Empty(variant, configuration);

        let initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("a.rs", "old");
        workspace.Apply(&initial).expect("a fresh present is always accepted");

        return workspace;
    }

    /// A plan with a single candidate that rewrites `a.rs` from `before` to `after`.
    fn Plan_Changing_A(before: &str, after: &str) -> CorrectionPlan
    {
        return CorrectionPlan::New(vec![CorrectionCandidate::New(
            "fix a",
            ChangeSet::Empty().With(Edit::New(
                "a.rs",
                Some(before.to_owned()),
                Some(after.to_owned()),
            )),
        )])
        .expect("a single candidate is a valid plan");
    }

    /// Stages, validates and commits `plan` against `base` in one step.
    fn Commit_Plan(plan: &CorrectionPlan, base: &mut Workspace) -> CommittedPlan
    {
        return plan
            .Stage(base)
            .expect("stages cleanly")
            .Validate(base)
            .expect("validates cleanly")
            .Commit(base)
            .expect("commits cleanly");
    }

    /// The invariant that makes rollback worth having: undoing a committed plan returns
    /// the workspace to exactly the snapshot it started from, byte for byte.
    #[test]
    fn Test_Committing_Then_Rolling_Back_Should_Return_To_The_Base_Snapshot()
    {
        let mut base = Base();
        let starting = base.Id();
        let plan = Plan_Changing_A("old", "new");
        let committed = Commit_Plan(&plan, &mut base);

        assert_ne!(base.Id(), starting, "the commit must have changed something");

        let after_rollback = committed.Rollback(&mut base).expect("rolls back cleanly");

        assert_eq!(after_rollback, starting);
        assert_eq!(base.Id(), starting);
        assert_eq!(base.Content_Of("a.rs"), Some(Content_Digest(b"old")));
    }

    #[test]
    fn Test_Rollback_After_The_Workspace_Moved_Should_Be_Refused()
    {
        let mut base = Base();
        let plan = Plan_Changing_A("old", "new");
        let committed = Commit_Plan(&plan, &mut base);

        let advance = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("b.rs", "other");
        base.Apply(&advance)
            .expect("an unrelated change still advances the workspace");

        let refusal = committed
            .Rollback(&mut base)
            .expect_err("the workspace moved since commit");

        assert!(matches!(refusal, crate::CorrectionError::Moved { .. }));
    }
}
