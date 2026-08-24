//! A plan checked once more, immediately before it is submitted through the door.

use crate::staged::Assert_Not_Moved;
use crate::{CommittedPlan, CorrectionError};
use nomos_contracts::SnapshotId;
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
        Assert_Not_Moved(self.base, live)?;

        live.Apply(&self.forward)?;

        return Ok(CommittedPlan::Of(self.base, live.Id(), self.reverse, evidence));
    }
}

#[cfg(test)]
mod tests
{
    use crate::{ChangeSet, CorrectionCandidate, CorrectionClass, CorrectionError, CorrectionPlan, Edit};
    use nomos_contracts::{ConfigurationId, Digest128, EvidenceClass, ProviderId};
    use nomos_model::{Content_Digest, Evidence};
    use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};

    /// What a caller with nothing stronger than its own judgment supplies.
    fn Agent_Judged() -> Evidence
    {
        return Evidence {
            class: EvidenceClass::AgentJudged,
            producer: ProviderId::New("test"),
            supporting: Vec::new(),
        };
    }

    fn Base() -> Workspace
    {
        let variant = BuildVariant::New("x86_64-unknown-none", "test", "fixed", Vec::<String>::new());
        let configuration = ConfigurationId::From_Digest(Digest128::From_Bytes([0x22; 16]));
        let mut workspace = Workspace::Empty(variant, configuration);

        let initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("a.rs", "old");
        workspace.Apply(&initial).expect("a fresh present is always accepted");

        return workspace;
    }

    #[test]
    fn Test_Committing_A_Validated_Plan_Should_Change_The_Workspace()
    {
        let mut base = Base();
        let plan = CorrectionPlan::New(vec![CorrectionCandidate::New(
            "fix a",
            ChangeSet::Empty().With(Edit::New("a.rs", Some("old".to_owned()), Some("new".to_owned()))),
            CorrectionClass::Mechanical,
            vec![],
        )])
        .expect("a single candidate is a valid plan");
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
        let mut base = Base();
        let plan = CorrectionPlan::New(vec![CorrectionCandidate::New(
            "fix a",
            ChangeSet::Empty().With(Edit::New("a.rs", Some("old".to_owned()), Some("new".to_owned()))),
            CorrectionClass::Mechanical,
            vec![],
        )])
        .expect("a single candidate is a valid plan");
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
}
