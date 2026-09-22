//! Fixtures shared by `committed_plan`, `staged_plan` and `validated_plan`'s own unit
//! tests -- each built the same base workspace and the same single-candidate plan
//! independently before this module existed to hold the one copy -- and, since
//! `compatibility`, `plan_access` and `wave_partition` arrived, the plans those three
//! judge against each other, which never touch a workspace at all.

use crate::{
    ChangeSet, CorrectionCandidate, CorrectionClass, CorrectionError, CorrectionPlan, DerivedProvenance, Edit, ReadWriteResolution,
    ReadWriteSet,
};
use nomos_contracts::{Assurance, ConfigurationId, Digest128, EvidenceClass, FactVariant, Guarantee, IncrementalGranularity, ProviderId};
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

/// One mechanical candidate that sets each of `paths` to `"x"` with no prior content
/// declared -- so its only read/write set is the artifact-tier floor `New` seeds, naming
/// exactly `paths`.
pub(crate) fn Candidate_Writing(paths: &[&str]) -> CorrectionCandidate
{
    let change = paths.iter().fold(ChangeSet::Empty(), |change, path| return change.With(Edit::New(*path, None, Some("x".to_owned()))));

    return CorrectionCandidate::New(format!("write {}", paths.join(", ")), change, CorrectionClass::Mechanical, vec![]);
}

/// `candidate`, declaring it also reads `path` at the artifact tier without writing it.
pub(crate) fn Reading(candidate: CorrectionCandidate, path: &str) -> CorrectionCandidate
{
    return candidate.With_Read(ReadWriteSet::Declared(ReadWriteResolution::Artifact, vec![path.to_owned()]));
}

/// A plan of exactly `candidate`. One candidate can be neither empty nor in conflict with
/// a sibling, so the constructor's refusal is not a case these fixtures can reach.
pub(crate) fn Plan_Of(candidate: CorrectionCandidate) -> CorrectionPlan
{
    return CorrectionPlan::New(vec![candidate]).expect("one candidate is a valid plan");
}

/// A plan of one candidate writing `paths`, the shape every compatibility case starts
/// from.
pub(crate) fn Plan_Writing(paths: &[&str]) -> CorrectionPlan
{
    return Plan_Of(Candidate_Writing(paths));
}

/// A dependency-tier set some provider derived, whose guarantee claims `completeness`
/// and is otherwise sound -- the one axis `plan_access` decides resolution on, varied
/// while everything else is held fixed.
pub(crate) fn Derived_Set_With_Completeness(completeness: Assurance) -> ReadWriteSet
{
    let guarantee = Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, completeness, IncrementalGranularity::File);
    let provenance = DerivedProvenance::New(ProviderId::New("nomos-lang-rust"), guarantee, "high", "invalidated when the dependency graph changes");

    return ReadWriteSet::Derived(ReadWriteResolution::Dependency, vec!["nomos-corrections -> nomos-workspace".to_owned()], provenance);
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

    #[test]
    fn Test_Candidate_Writing_Should_Touch_Exactly_The_Given_Paths()
    {
        let candidate = Candidate_Writing(&["a.rs", "b.rs"]);

        assert_eq!(candidate.Change().Touched(), std::collections::BTreeSet::from(["a.rs", "b.rs"]));
        assert!(candidate.Reads().is_empty());
    }

    #[test]
    fn Test_Reading_Should_Declare_One_Artifact_Tier_Read()
    {
        let candidate = Reading(Candidate_Writing(&["b.rs"]), "a.rs");

        assert_eq!(candidate.Reads(), [ReadWriteSet::Declared(ReadWriteResolution::Artifact, vec!["a.rs".to_owned()])]);
    }

    #[test]
    fn Test_Plan_Writing_Should_Hold_One_Candidate()
    {
        assert_eq!(Plan_Writing(&["a.rs"]).Candidates().len(), 1);
    }

    #[test]
    fn Test_Derived_Set_With_Completeness_Should_Vary_Only_That_Axis()
    {
        let sound = Derived_Set_With_Completeness(Assurance::Sound);
        let unknown = Derived_Set_With_Completeness(Assurance::Unknown);

        assert_eq!(sound.Entries(), unknown.Entries());
        assert_eq!(sound.Derived_Provenance().map(|provenance| return provenance.Guarantee().completeness), Some(Assurance::Sound));
        assert_eq!(unknown.Derived_Provenance().map(|provenance| return provenance.Guarantee().completeness), Some(Assurance::Unknown));
    }
}
