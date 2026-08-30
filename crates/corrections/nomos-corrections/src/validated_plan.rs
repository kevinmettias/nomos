//! A plan checked once more, immediately before it is submitted through the door.

use crate::{CommittedPlan, CorrectionError};
use nomos_contracts::{MutationClass, SnapshotId};
use nomos_model::Evidence;
use nomos_workspace::{Workspace, WorkspaceChangeSet};

/// A plan that has been checked against a live workspace and is ready to commit.
///
/// Separate from [`crate::StagedPlan`] so that "checked" and "checked most recently" are
/// two different facts a caller can hold apart — a plan can be staged once and validated
/// again immediately before commit, with nothing else in between able to mistake one
/// check for the other.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedPlan
{
    base: SnapshotId,
    forward: WorkspaceChangeSet,
    reverse: WorkspaceChangeSet,
}

impl ValidatedPlan
{
    pub(crate) fn Of(base: SnapshotId, forward: WorkspaceChangeSet, reverse: WorkspaceChangeSet) -> Self
    {
        return Self {
            base,
            forward,
            reverse,
        };
    }

    /// `AGT-EXEC-001`'s capability-class question, answered for this operation:
    /// `nomos_contracts::MutationClass::Validate`, the same class
    /// `StagedPlan::Validate` -- the method that produces a value of this type --
    /// belongs to. `Self::Commit`, below, is the actual `MutationClass::Apply` step;
    /// this value's own class is what produced it, not what it goes on to do.
    #[must_use]
    pub const fn Mutation_Class() -> MutationClass
    {
        return MutationClass::Validate;
    }

    /// Submits this plan's forward change through `live`'s one door.
    ///
    /// `evidence` is not judged here and does not change what this method refuses -- a
    /// plan whose workspace moved is refused regardless of what backs the caller's claim
    /// that committing is warranted. It is carried onto the resulting [`CommittedPlan`]
    /// unchanged, so a caller with nothing stronger than its own judgment can still commit,
    /// honestly labelled [`nomos_model::EvidenceClass::AgentJudged`], rather than the
    /// alternative this parameter exists to end: a commit backed by nothing, indistinguishable
    /// from one backed by a passing test suite.
    ///
    /// # Errors
    ///
    /// Returns [`CorrectionError::Moved`] if `live` has moved since this plan was
    /// validated, or [`CorrectionError::Workspace`] if the workspace door itself refuses
    /// the change.
    pub fn Commit(self, live: &mut Workspace, evidence: Evidence) -> Result<CommittedPlan, CorrectionError>
    {
        use crate::staged_plan::Assert_Not_Moved;

        Assert_Not_Moved(self.base, live)?;

        live.Apply(&self.forward)?;

        return Ok(CommittedPlan::Of(self.base, live.Id(), self.reverse, evidence));
    }
}

#[cfg(test)]
mod tests
{
    use crate::test_support::{Agent_Judged, Base, Plan_Changing_A};
    use crate::{CorrectionError, ValidatedPlan};
    use nomos_contracts::MutationClass;
    use nomos_model::Content_Digest;
    use nomos_workspace::{ChangeSource, WorkspaceChangeSet};

    const CONFIGURATION_SEED_BYTE: u8 = 0x22;

    #[test]
    fn Test_Committing_A_Validated_Plan_Should_Change_The_Workspace()
    {
        let mut base = Base(CONFIGURATION_SEED_BYTE);
        let plan = Plan_Changing_A("old", "new");
        let before = base.Id();
        let validated = plan
            .Stage(&base)
            .expect("stages cleanly")
            .Validate(&base)
            .expect("validates cleanly");

        let committed = validated.Commit(&mut base, Agent_Judged()).expect("commits cleanly");

        assert_ne!(before, base.Id());
        assert_eq!(base.Content_Of("a.rs"), Some(Content_Digest(b"new")));
        assert_eq!(committed.After(), base.Id());
    }

    #[test]
    fn Test_Committing_After_The_Workspace_Moved_Should_Be_Refused()
    {
        let mut base = Base(CONFIGURATION_SEED_BYTE);
        let plan = Plan_Changing_A("old", "new");
        let validated = plan
            .Stage(&base)
            .expect("stages cleanly")
            .Validate(&base)
            .expect("validates cleanly");

        let advance = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("b.rs", "other");
        base.Apply(&advance)
            .expect("an unrelated change still advances the workspace");

        let refusal = validated
            .Commit(&mut base, Agent_Judged())
            .expect_err("the workspace moved since validation");

        assert!(matches!(refusal, CorrectionError::Moved { .. }));
    }

    #[test]
    fn Test_A_Validated_Plan_Belongs_To_The_Validate_Mutation_Class()
    {
        assert_eq!(ValidatedPlan::Mutation_Class(), MutationClass::Validate);
    }
}
