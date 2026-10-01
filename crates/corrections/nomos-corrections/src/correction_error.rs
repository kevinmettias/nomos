//! Every way a correction refuses.

use nomos_contracts::{Digest128, SnapshotId};
use nomos_workspace::WorkspaceError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorrectionError
{
    /// A plan with no candidates in it.
    Vacuous,
    /// Two candidates in one plan touch the same path.
    ///
    /// Refused rather than resolved by order, for the reason `WorkspaceError::Conflicting`
    /// is: a plan that changes one path two ways has no correct reading, and picking one
    /// silently would make the result depend on the order candidates happened to be given.
    Conflicting
    {
        paths: Vec<String>,
    },
    /// A candidate's declared prior content does not match what a workspace currently
    /// holds at that path.
    ///
    /// Caught at staging, before anything is submitted through the door: a candidate built
    /// against a state that has since moved is not safe to apply, whether or not its
    /// result would happen to be harmless.
    StaleCandidate
    {
        path: String,
        expected: Option<Digest128>,
        found: Option<Digest128>,
    },
    /// A workspace moved between staging, validating, committing or rolling back a plan.
    Moved
    {
        expected: SnapshotId,
        found: SnapshotId,
    },
    /// The workspace door itself refused the change.
    Workspace(WorkspaceError),
}

impl core::fmt::Display for CorrectionError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Vacuous => write!(
                formatter,
                "a correction plan with no candidates in it. This is refused rather than \
                 ignored because it is almost always a loop that iterated zero times"
            ),
            Self::Conflicting { paths } => write!(
                formatter,
                "these paths are touched by more than one candidate in the same plan: \
                 {paths:?}. There is no correct reading of a path two candidates both \
                 change, and choosing one would make the result depend on candidate order"
            ),
            Self::StaleCandidate {
                path,
                expected,
                found,
            } => write!(
                formatter,
                "`{path}` was expected to hold {expected:?} but the workspace holds {found:?}. \
                 The candidate was built against a state this workspace has since moved past"
            ),
            Self::Moved { expected, found } => write!(
                formatter,
                "the workspace moved from {expected} to {found} since this plan was staged. \
                 A plan is only safe to carry forward against the state it was checked against"
            ),
            Self::Workspace(error) => error.fmt(formatter),
        };
    }
}

impl std::error::Error for CorrectionError
{}

impl From<WorkspaceError> for CorrectionError
{
    fn from(error: WorkspaceError) -> Self
    {
        return Self::Workspace(error);
    }
}
