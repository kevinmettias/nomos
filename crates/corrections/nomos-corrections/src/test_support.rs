//! Fixtures shared by `committed_plan`, `staged_plan` and `validated_plan`'s own unit
//! tests -- each built the same base workspace and the same single-candidate plan
//! independently before this module existed to hold the one copy.

use crate::{ChangeSet, CorrectionCandidate, CorrectionClass, CorrectionError, CorrectionPlan, Edit};
use nomos_contracts::{ConfigurationId, Digest128, EvidenceClass, ProviderId};
use nomos_model::Evidence;
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet, WorkspaceError};

/// A workspace with one present file, `a.rs` containing `"old"`, under a caller-chosen
/// configuration seed byte -- distinct per call site so a snapshot digest collision
/// between two unrelated tests is never mistaken for a real property of the code under
/// test.
///
/// # Errors
///
/// Returns whatever [`Workspace::Apply`] refuses. Applying one `Present` to an empty
/// workspace is not expected to be refused; the error is in the signature because the
/// operation it wraps is fallible, not because this fixture anticipates a failure.
pub(crate) fn Workspace_With_One_File(configuration_seed_byte: u8) -> Result<Workspace, WorkspaceError>
{
    let variant = BuildVariant::New("x86_64-unknown-none", "test", "fixed", Vec::<String>::new());
    let configuration =
        ConfigurationId::From_Digest(Digest128::From_Bytes([configuration_seed_byte; Digest128::BYTE_LENGTH]));
    let mut workspace = Workspace::Empty(variant, configuration);

    let initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("a.rs", "old");
    workspace.Apply(&initial)?;

    return Ok(workspace);
}

/// The rewrite [`Plan_Changing_A`] is asked to build: the text `a.rs` holds and the text
/// it should hold afterwards.
///
/// Named fields rather than two adjacent `&str` positions, which is what a call site
/// reading `Plan_Changing_A(before, after)` could transpose without the compiler
/// objecting.
pub(crate) struct Rewrite<'text>
{
    /// The text `a.rs` is expected to hold before the plan runs.
    pub from: &'text str,
    /// The text the plan should leave in `a.rs`.
    pub to: &'text str,
}

/// A plan with a single candidate that rewrites `a.rs` from `rewrite.from` to `rewrite.to`.
///
/// # Errors
///
/// Returns whatever [`CorrectionPlan::New`] refuses. One candidate touching one path can
/// be neither empty nor in conflict with a sibling, so this fixture expects no refusal;
/// the error is in the signature because the plan's own constructor is fallible.
pub(crate) fn Plan_Changing_A(rewrite: Rewrite<'_>) -> Result<CorrectionPlan, CorrectionError>
{
    return CorrectionPlan::New(vec![CorrectionCandidate::New(
        "fix a",
        ChangeSet::Empty().With(Edit::New("a.rs", Some(rewrite.from.to_owned()), Some(rewrite.to.to_owned()))),
        CorrectionClass::Mechanical,
        vec![],
    )]);
}

/// What a caller with nothing stronger than its own judgment supplies.
pub(crate) fn Agent_Judged() -> Evidence
{
    return Evidence {
        class: EvidenceClass::AgentJudged,
        producer: ProviderId::New("test"),
        supporting: Vec::new(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Base_Should_Start_With_The_Configured_File_Present()
    {
        let workspace = Workspace_With_One_File(0x01).expect("one Present applies to an empty workspace");

        assert_eq!(workspace.Content_Of("a.rs"), Some(nomos_model::Content_Digest(b"old")));
    }

    #[test]
    fn Test_Plan_Changing_A_Should_Build_A_Plan_That_Rewrites_The_File()
    {
        let plan = Plan_Changing_A(Rewrite {
            from: "old",
            to: "new",
        })
        .expect("one candidate touching one path is a valid plan");

        assert_eq!(plan.Candidates().len(), 1);
    }

    #[test]
    fn Test_Agent_Judged_Should_Report_The_Agent_Judged_Evidence_Class()
    {
        let evidence = Agent_Judged();

        assert_eq!(evidence.class, EvidenceClass::AgentJudged);
    }
}
