//! A plan checked against a live workspace, and ready to be validated.

use crate::{CorrectionError, CorrectionPlan, Edit, ValidatedPlan};
use nomos_contracts::SnapshotId;
use nomos_model::Content_Digest;
use nomos_workspace::{Change, ChangeSource, Workspace, WorkspaceChangeSet};

/// A plan whose candidates have each been checked against the workspace they were staged
/// over.
///
/// Staging is where a plan first touches something live, and where a candidate built
/// against content that has since moved is caught — before either changeset is ever
/// submitted through the workspace's one door.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StagedPlan
{
    base: SnapshotId,
    forward: WorkspaceChangeSet,
    reverse: WorkspaceChangeSet,
}

impl StagedPlan
{
    pub(crate) fn Of(plan: &CorrectionPlan, base: &Workspace) -> Result<Self, CorrectionError>
    {
        let mut forward = WorkspaceChangeSet::From(ChangeSource::Correction);
        let mut reverse = WorkspaceChangeSet::From(ChangeSource::Correction);

        for candidate in plan.Candidates()
        {
            for edit in candidate.Change().Edits()
            {
                Assert_Not_Stale(base, edit)?;

                forward = Apply_Onto(forward, edit.Forward());
                reverse = Apply_Onto(reverse, edit.Reverse());
            }
        }

        return Ok(Self {
            base: base.Id(),
            forward,
            reverse,
        });
    }

    #[must_use]
    pub const fn Base(&self) -> SnapshotId
    {
        return self.base;
    }

    /// Checks the plan is still safe to commit: the workspace has not moved since staging.
    ///
    /// # Errors
    ///
    /// Returns [`CorrectionError::Moved`] if `live`'s current snapshot differs from the one
    /// this plan was staged against.
    pub fn Validate(&self, live: &Workspace) -> Result<ValidatedPlan, CorrectionError>
    {
        Assert_Not_Moved(self.base, live)?;

        return Ok(ValidatedPlan::Of(self.base, self.forward.clone(), self.reverse.clone()));
    }
}

/// A change set built one change at a time; there is no bulk constructor to fold over.
fn Apply_Onto(set: WorkspaceChangeSet, change: Change) -> WorkspaceChangeSet
{
    return match change
    {
        Change::Present { path, content } => set.Present(path, content),
        Change::Absent { path } => set.Absent(path),
    };
}

fn Assert_Not_Stale(base: &Workspace, edit: &Edit) -> Result<(), CorrectionError>
{
    let expected = edit.Before().map(|before| return Content_Digest(before.as_bytes()));
    let found = base.Content_Of(edit.Path());

    if expected != found
    {
        return Err(CorrectionError::StaleCandidate {
            path: edit.Path().to_owned(),
            expected,
            found,
        });
    }

    return Ok(());
}

/// Shared by [`StagedPlan::Validate`], [`crate::ValidatedPlan::Commit`] and
/// [`crate::CommittedPlan::Rollback`] — every step that hands a plan to a live workspace
/// checks it has not moved past the state the previous step saw.
pub(crate) fn Assert_Not_Moved(expected: SnapshotId, live: &Workspace) -> Result<(), CorrectionError>
{
    let found = live.Id();

    if found != expected
    {
        return Err(CorrectionError::Moved { expected, found });
    }

    return Ok(());
}

#[cfg(test)]
mod tests
{
    use crate::{ChangeSet, CorrectionCandidate, CorrectionError, CorrectionPlan, Edit};
    use nomos_contracts::{ConfigurationId, Digest128};
    use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};

    fn Base() -> Workspace
    {
        let variant = BuildVariant::New("x86_64-unknown-none", "test", "fixed", Vec::<String>::new());
        let configuration = ConfigurationId::From_Digest(Digest128::From_Bytes([0x11; 16]));
        let mut workspace = Workspace::Empty(variant, configuration);

        let initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("a.rs", "old");
        workspace.Apply(&initial).expect("a fresh present is always accepted");

        return workspace;
    }

    #[test]
    fn Test_Staging_Against_The_Content_It_Was_Built_Over_Should_Succeed()
    {
        let base = Base();
        let plan = CorrectionPlan::New(vec![CorrectionCandidate::New(
            "fix a",
            ChangeSet::Empty().With(Edit::New("a.rs", Some("old".to_owned()), Some("new".to_owned()))),
        )])
        .expect("a single candidate is a valid plan");

        let staged = plan.Stage(&base).expect("the candidate's prior content matches");

        assert_eq!(staged.Base(), base.Id());
    }

    #[test]
    fn Test_Staging_Against_Stale_Content_Should_Be_Refused()
    {
        let base = Base();
        let plan = CorrectionPlan::New(vec![CorrectionCandidate::New(
            "fix a",
            ChangeSet::Empty().With(Edit::New(
                "a.rs",
                Some("not what is there".to_owned()),
                Some("new".to_owned()),
            )),
        )])
        .expect("a single candidate is a valid plan");

        let refusal = plan.Stage(&base).expect_err("the candidate's prior content is wrong");

        assert!(matches!(refusal, CorrectionError::StaleCandidate { .. }));
    }

    #[test]
    fn Test_Validating_An_Unmoved_Workspace_Should_Succeed()
    {
        let base = Base();
        let plan = CorrectionPlan::New(vec![CorrectionCandidate::New(
            "fix a",
            ChangeSet::Empty().With(Edit::New("a.rs", Some("old".to_owned()), Some("new".to_owned()))),
        )])
        .expect("a single candidate is a valid plan");
        let staged = plan.Stage(&base).expect("stages cleanly");

        assert!(staged.Validate(&base).is_ok());
    }

    #[test]
    fn Test_Validating_A_Moved_Workspace_Should_Be_Refused()
    {
        let mut base = Base();
        let plan = CorrectionPlan::New(vec![CorrectionCandidate::New(
            "fix a",
            ChangeSet::Empty().With(Edit::New("a.rs", Some("old".to_owned()), Some("new".to_owned()))),
        )])
        .expect("a single candidate is a valid plan");
        let staged = plan.Stage(&base).expect("stages cleanly");

        let advance = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("b.rs", "other");
        base.Apply(&advance)
            .expect("an unrelated change still advances the workspace");

        let refusal = staged.Validate(&base).expect_err("the workspace moved since staging");

        assert!(matches!(refusal, CorrectionError::Moved { .. }));
    }
}
