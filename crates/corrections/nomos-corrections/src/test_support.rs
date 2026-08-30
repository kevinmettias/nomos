//! Fixtures shared by `committed_plan`, `staged_plan` and `validated_plan`'s own unit
//! tests -- each built the same base workspace and the same single-candidate plan
//! independently before this module existed to hold the one copy.

use crate::{ChangeSet, CorrectionCandidate, CorrectionClass, CorrectionPlan, Edit};
use nomos_contracts::{ConfigurationId, Digest128, EvidenceClass, ProviderId};
use nomos_model::Evidence;
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};

/// A workspace with one present file, `a.rs` containing `"old"`, under a caller-chosen
/// configuration seed byte -- distinct per call site so a snapshot digest collision
/// between two unrelated tests is never mistaken for a real property of the code under
/// test.
pub(crate) fn Base(configuration_seed_byte: u8) -> Workspace
{
    let variant = BuildVariant::New("x86_64-unknown-none", "test", "fixed", Vec::<String>::new());
    let configuration =
        ConfigurationId::From_Digest(Digest128::From_Bytes([configuration_seed_byte; Digest128::BYTE_LENGTH]));
    let mut workspace = Workspace::Empty(variant, configuration);

    let initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("a.rs", "old");
    workspace.Apply(&initial).expect("a fresh present is always accepted");

    return workspace;
}

/// A plan with a single candidate that rewrites `a.rs` from `before` to `after`.
pub(crate) fn Plan_Changing_A(before: &str, after: &str) -> CorrectionPlan
{
    return CorrectionPlan::New(vec![CorrectionCandidate::New(
        "fix a",
        ChangeSet::Empty().With(Edit::New("a.rs", Some(before.to_owned()), Some(after.to_owned()))),
        CorrectionClass::Mechanical,
        vec![],
    )])
    .expect("a single candidate is a valid plan");
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
        let workspace = Base(0x01);

        assert_eq!(workspace.Content_Of("a.rs"), Some(nomos_model::Content_Digest(b"old")));
    }

    #[test]
    fn Test_Plan_Changing_A_Should_Build_A_Plan_That_Rewrites_The_File()
    {
        let plan = Plan_Changing_A("old", "new");

        assert_eq!(plan.Candidates().len(), 1);
    }

    #[test]
    fn Test_Agent_Judged_Should_Report_The_Agent_Judged_Evidence_Class()
    {
        let evidence = Agent_Judged();

        assert_eq!(evidence.class, EvidenceClass::AgentJudged);
    }
}
