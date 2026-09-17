//! `nomos-corrections`'s own `#[cfg(test)]` unit tests -- in `staged_plan.rs`,
//! `validated_plan.rs`, `committed_plan.rs` and `test_support.rs` -- already build a real
//! `nomos_workspace::Workspace`, hand a real `nomos_model::Evidence` to `Commit`, and read
//! back real `nomos_contracts::Digest128`/`MutationClass`/`SnapshotId` values. But those are
//! compiled INTO the crate, where they can also see its own private items, so they prove
//! the two sides work when one of them is opened up rather than proving the PUBLIC contract
//! holds. Compiled outside `src/`, this file can reach `nomos_contracts`, `nomos_model` and
//! `nomos_workspace` only through this crate's own public lifecycle --
//! [`CorrectionPlan`]/[`StagedPlan`]/[`ValidatedPlan`]/[`CommittedPlan`] -- the same view a
//! real consumer has.

use nomos_contracts::{ConfigurationId, Digest128, EvidenceClass, MutationClass, ProviderId, SnapshotId};
use nomos_corrections::{
    ChangeSet, CommittedPlan, CorrectionCandidate, CorrectionClass, CorrectionError, CorrectionId, CorrectionPlan, Edit, StagedPlan,
    ValidatedPlan,
};
use nomos_model::{Content_Digest, Evidence};
use nomos_workspace::{BuildVariant, ChangeSource, Workspace, WorkspaceChangeSet};

const CONFIGURATION_SEED_BYTE_ROUND_TRIP: u8 = 0x51;
const CONFIGURATION_SEED_BYTE_STALE: u8 = 0x52;
const ARBITRARY_DIGEST_BYTE: u8 = 0x53;

/// A real workspace built the way a real consumer would: a genuine
/// `nomos_workspace::BuildVariant` and `nomos_contracts::ConfigurationId`, advanced through
/// the one door every source -- a correction included -- submits through, with `a.rs`
/// present holding `"old"`.
fn Real_Workspace(configuration_seed_byte: u8) -> Workspace
{
    let variant = BuildVariant::New("x86_64-unknown-none", "test", "fixed", Vec::<String>::new());
    let configuration = ConfigurationId::From_Digest(Digest128::From_Bytes([configuration_seed_byte; Digest128::BYTE_LENGTH]));
    let mut workspace = Workspace::Empty(variant, configuration);

    let initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("a.rs", "old");
    workspace.Apply(&initial).expect("Workspace::Apply admits a Present whose path holds no prior content");

    return workspace;
}

/// One mechanical candidate rewriting `a.rs` from `prior` to `"new"`, wrapped in the plan a
/// caller would submit it as.
fn Rewrite_Plan(prior: &str) -> CorrectionPlan
{
    let edit = Edit::New("a.rs", Some(prior.to_owned()), Some("new".to_owned()));
    let candidate = CorrectionCandidate::New("rewrite a.rs", ChangeSet::Empty().With(edit), CorrectionClass::Mechanical, vec![]);

    return CorrectionPlan::New(vec![candidate]).expect("one candidate is a valid plan");
}

/// Stage, validate and commit `plan` against `workspace`, asserting at each seam that the
/// staged plan was built against `starting` and that the mutation class it declares is the one
/// the stage carries.
fn Advance_Plan_Through_The_Seams(plan: &CorrectionPlan, workspace: &mut Workspace, starting: SnapshotId) -> CommittedPlan
{
    let staged = plan.Stage(workspace).expect("the candidate's declared prior content matches the real workspace");
    assert_eq!(staged.Base(), starting, "a plan stages against the state it was previewed on");
    assert_eq!(StagedPlan::Mutation_Class(), MutationClass::Validate);

    let validated = staged.Validate(workspace).expect("the workspace has not moved since staging");
    assert_eq!(ValidatedPlan::Mutation_Class(), MutationClass::Validate);

    return validated
        .Commit(workspace, Agent_Judged())
        .expect("commits cleanly against the real workspace");
}

/// What a caller with nothing stronger than its own judgment supplies to
/// `ValidatedPlan::Commit`.
fn Agent_Judged() -> Evidence
{
    return Evidence {
        class: EvidenceClass::AgentJudged,
        producer: ProviderId::New("integration-seam-test"),
        supporting: Vec::new(),
    };
}

/// The whole `Preview -> Stage -> Validate -> Commit -> Rollback` lifecycle, driven by a
/// real `nomos_workspace::Workspace` and a real `nomos_model::Evidence`, reading back real
/// `nomos_contracts::SnapshotId` and `MutationClass` values at every step -- the three seams
/// this crate's own internal tests exercise, proved here through nothing but the public API
/// a real consumer has.
#[test]
fn Test_The_Full_Lifecycle_Round_Trips_A_Real_Workspace_Through_The_Public_Api()
{
    let mut workspace = Real_Workspace(CONFIGURATION_SEED_BYTE_ROUND_TRIP);
    let starting = workspace.Id();
    let plan = Rewrite_Plan("old");

    assert!(!plan.Preview().Rendered().is_empty(), "a real preview renders something for a real edit");

    let committed = Advance_Plan_Through_The_Seams(&plan, &mut workspace, starting);
    assert_ne!(committed.After(), starting, "a real commit must have actually advanced the workspace");
    assert_eq!(
        workspace.Content_Of("a.rs"),
        Some(Content_Digest(b"new")),
        "nomos_model's own digest algorithm is what the workspace now reports for a.rs"
    );
    assert_eq!(committed.Evidence(), &Agent_Judged());
    assert_eq!(CommittedPlan::Mutation_Class(), MutationClass::Apply);
    assert_eq!(CommittedPlan::ROLLBACK_MUTATION_CLASS, MutationClass::Rollback);

    let after_rollback = committed.Rollback(&mut workspace).expect("rolls back cleanly against the real workspace");
    assert_eq!(after_rollback, starting);
    assert_eq!(workspace.Id(), starting);
    assert_eq!(workspace.Content_Of("a.rs"), Some(Content_Digest(b"old")));
}

/// `CorrectionError::StaleCandidate`'s `expected`/`found` fields carry real
/// `nomos_contracts::Digest128` values computed by the exact algorithm this crate hands to
/// `nomos_model::Content_Digest` -- checked here by computing the same digest independently
/// from outside the crate, over a real `nomos_workspace::Workspace`, rather than only
/// checking the refusal's shape.
#[test]
fn Test_Staging_Against_Stale_Content_Reports_The_Real_Digest_Of_What_Is_There()
{
    let workspace = Real_Workspace(CONFIGURATION_SEED_BYTE_STALE);
    let plan = Rewrite_Plan("not what is there");

    let refusal = plan.Stage(&workspace).expect_err("the declared prior content does not match the real workspace");

    match refusal
    {
        CorrectionError::StaleCandidate { path, expected, found } =>
        {
            assert_eq!(path, "a.rs");
            assert_eq!(expected, Some(Content_Digest(b"not what is there")));
            assert_eq!(found, Some(Content_Digest(b"old")));
        }
        other => panic!("a mismatched prior content must refuse as StaleCandidate, got {other:?}"),
    }
}

/// `CorrectionId` carries a real `nomos_contracts::Digest128` rather than an opaque token
/// of its own -- constructing one from a digest and reading it back must round-trip the
/// exact bytes.
#[test]
fn Test_A_Correction_Ids_Digest_Round_Trips_A_Real_Digest128()
{
    let digest = Digest128::From_Bytes([ARBITRARY_DIGEST_BYTE; Digest128::BYTE_LENGTH]);

    let id = CorrectionId::From_Digest(digest);

    assert_eq!(id.Digest(), digest);
}
