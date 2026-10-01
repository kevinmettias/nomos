//! A plan that has been submitted through the door, and the reverse that undoes it.

use crate::{CorrectionError, RollbackBoundary};
use nomos_contracts::{MutationClass, SnapshotId};
use nomos_model::Evidence;
use nomos_workspace::{Workspace, WorkspaceChangeSet};

/// A plan whose forward change has been applied to a live workspace.
///
/// Carries the exact reverse of what it applied, built from the same edits at staging
/// time — not recomputed from whatever the workspace holds when rollback is asked for, so
/// rollback undoes what this plan did rather than whatever the paths currently say. Also
/// carries the evidence [`crate::ValidatedPlan::Commit`] was given for why committing was
/// warranted — `AGT-EXEC-004`'s own requirement, `OD-CORRECTIONS-002` decided, is that this
/// claim is never silent, not that it is always strong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommittedPlan
{
    base: SnapshotId,
    after: SnapshotId,
    reverse: WorkspaceChangeSet,
    evidence: Evidence,
}

impl CommittedPlan
{
    pub(crate) fn Of(base: SnapshotId, after: SnapshotId, reverse: WorkspaceChangeSet, evidence: Evidence) -> Self
    {
        return Self {
            base,
            after,
            reverse,
            evidence,
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

    /// What the committing caller said backed its claim that committing was warranted.
    #[must_use]
    pub fn Evidence(&self) -> &Evidence
    {
        return &self.evidence;
    }

    /// `AGT-EXEC-001`'s capability-class question, answered for this value's own
    /// history: `nomos_contracts::MutationClass::Apply`, the class
    /// `ValidatedPlan::Commit` -- the method that produced this value -- belongs to.
    /// [`Self::ROLLBACK_MUTATION_CLASS`] is the separate answer for `Self::Rollback`,
    /// the operation this value is the *receiver* of rather than the *result* of.
    #[must_use]
    pub const fn Mutation_Class() -> MutationClass
    {
        return MutationClass::Apply;
    }

    /// The mutation class [`Self::Rollback`] itself belongs to --
    /// `nomos_contracts::MutationClass::Rollback`. An associated constant rather than a
    /// method on `self`, because `Rollback` consumes `self` and returns a bare
    /// `SnapshotId` with nowhere to hang an instance method that describes the call
    /// that already happened.
    pub const ROLLBACK_MUTATION_CLASS: MutationClass = MutationClass::Rollback;

    /// `COR-EXEC-006`'s declared reversal kind for [`Self::Rollback`]: always
    /// [`RollbackBoundary::Exact`]. Provable rather than declared by a caller, because
    /// `Rollback` restores the exact reverse changeset recorded at commit time --
    /// `Test_Committing_Then_Rolling_Back_Should_Return_To_The_Base_Snapshot`, below, is
    /// what makes that a fact about this implementation rather than an aspiration.
    #[must_use]
    pub const fn Rollback_Boundary() -> RollbackBoundary
    {
        return RollbackBoundary::Exact;
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
        use crate::staged_plan::Assert_Not_Moved;

        Assert_Not_Moved(self.after, live)?;

        live.Apply(&self.reverse)?;

        return Ok(live.Id());
    }
}

#[cfg(test)]
mod tests
{
    use crate::test_support::{Agent_Judged, Workspace_With_One_File, Plan_Changing_A, Rewrite};
    use crate::{CommittedPlan, CorrectionPlan};
    use nomos_contracts::MutationClass;
    use nomos_model::Content_Digest;
    use nomos_workspace::{ChangeSource, Workspace, WorkspaceChangeSet};

    const CONFIGURATION_SEED_BYTE: u8 = 0x33;

    /// The invariant that makes rollback worth having: undoing a committed plan returns
    /// the workspace to exactly the snapshot it started from, byte for byte.
    #[test]
    fn Test_Committing_Then_Rolling_Back_Should_Return_To_The_Base_Snapshot()
    {
        let mut base = Workspace_With_One_File(CONFIGURATION_SEED_BYTE).expect("one Present applies to an empty workspace");
        let starting = base.Id();
        let plan = Plan_Changing_A(Rewrite { from: "old", to: "new" }).expect("one candidate touching one path is a valid plan");
        let committed = Commit_Plan(&plan, &mut base);

        assert_ne!(base.Id(), starting, "the commit must have changed something");

        let after_rollback = committed.Rollback(&mut base).expect("rolls back cleanly");

        assert_eq!(after_rollback, starting);
        assert_eq!(base.Id(), starting);
        assert_eq!(base.Content_Of("a.rs"), Some(Content_Digest(b"old")));
    }

    /// `OD-CORRECTIONS-002`'s own reason for existing: a committed plan does not lose what
    /// backed the claim that committing was warranted.
    #[test]
    fn Test_A_Committed_Plan_Should_Carry_Its_Evidence()
    {
        let mut base = Workspace_With_One_File(CONFIGURATION_SEED_BYTE).expect("one Present applies to an empty workspace");
        let plan = Plan_Changing_A(Rewrite { from: "old", to: "new" }).expect("one candidate touching one path is a valid plan");
        let committed = Commit_Plan(&plan, &mut base);

        assert_eq!(committed.Evidence(), &Agent_Judged());
    }

    #[test]
    fn Test_Rollback_After_The_Workspace_Moved_Should_Be_Refused()
    {
        let mut base = Workspace_With_One_File(CONFIGURATION_SEED_BYTE).expect("one Present applies to an empty workspace");
        let plan = Plan_Changing_A(Rewrite { from: "old", to: "new" }).expect("one candidate touching one path is a valid plan");
        let committed = Commit_Plan(&plan, &mut base);

        let advance = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("b.rs", "other");
        base.Apply(&advance)
            .expect("an unrelated change still advances the workspace");

        let refusal = committed
            .Rollback(&mut base)
            .expect_err("the workspace moved since commit");

        assert!(matches!(refusal, crate::CorrectionError::Moved { .. }));
    }

    #[test]
    fn Test_A_Committed_Plan_Belongs_To_The_Apply_Mutation_Class_And_Names_Rollbacks_Class()
    {
        assert_eq!(CommittedPlan::Mutation_Class(), MutationClass::Apply);
        assert_eq!(CommittedPlan::ROLLBACK_MUTATION_CLASS, MutationClass::Rollback);
    }

    #[test]
    fn Test_A_Committed_Plan_Declares_An_Exact_Rollback_Boundary()
    {
        assert_eq!(CommittedPlan::Rollback_Boundary(), crate::RollbackBoundary::Exact);
    }

    #[test]
    fn Test_Of_Should_Assemble_A_Committed_Plan_From_Its_Parts()
    {
        let mut base = Workspace_With_One_File(CONFIGURATION_SEED_BYTE).expect("one Present applies to an empty workspace");
        let starting = base.Id();
        let plan = Plan_Changing_A(Rewrite { from: "old", to: "new" }).expect("one candidate touching one path is a valid plan");

        let committed = Commit_Plan(&plan, &mut base);

        assert_eq!(committed.Base(), starting);
    }

    #[test]
    fn Test_After_Should_Report_The_Snapshot_Once_The_Change_Was_Applied()
    {
        let mut base = Workspace_With_One_File(CONFIGURATION_SEED_BYTE).expect("one Present applies to an empty workspace");
        let plan = Plan_Changing_A(Rewrite { from: "old", to: "new" }).expect("one candidate touching one path is a valid plan");

        let committed = Commit_Plan(&plan, &mut base);

        assert_eq!(committed.After(), base.Id());
    }

    /// Stages, validates and commits `plan` against `base` in one step.
    fn Commit_Plan(plan: &CorrectionPlan, base: &mut Workspace) -> CommittedPlan
    {
        return plan
            .Stage(base)
            .expect("stages cleanly")
            .Validate(base)
            .expect("validates cleanly")
            .Commit(base, Agent_Judged())
            .expect("commits cleanly");
    }
}
