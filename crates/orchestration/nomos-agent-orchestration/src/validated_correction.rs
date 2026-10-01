//! Driving an agent's `WorkResult.plan`, when there is one, through the correction
//! lifecycle `nomos-corrections` owns.
//!
//! `P40-CORRECTIONS-CANONICAL-SEAM` gave `nomos-cli`'s own `correct.rs` a canonical seam
//! for a rule-produced candidate; nothing carried an *agent*-produced one the same way. A
//! [`WorkResult`]'s own `plan` field has been a real `nomos_corrections::CorrectionPlan`
//! since `P42-EXECUTOR-WORKRESULT-IMPLEMENTATION-4`, but nothing downstream ever read it --
//! an agent run could produce a plan and nothing would ever stage, validate or commit it.
//!
//! [`Run_Validated_Correction`] is the whole lifecycle in one call: `Preview`, then
//! `Stage`, then `Validate`, then `Commit`, against whatever [`Workspace`] the caller
//! already holds. It adds no judgment of its own -- a refusal is a
//! [`CorrectionError`] unchanged, and `nomos-corrections` itself is what guarantees
//! `workspace` is never mutated until `Commit`'s own internal check passes; this seam does
//! not re-derive that guarantee, only carries a plan to the calls that already hold it.
//!
//! A result with no plan -- `OD-CONTRACTS-003`'s judgment-only case -- is not an error:
//! [`ValidatedCorrectionOutcome::NoPlan`] says so, distinctly from a plan that was proposed
//! and then refused.

use crate::ValidatedCorrectionOutcome;
use nomos_agent_contracts::WorkResult;
use nomos_corrections::CorrectionError;
use nomos_model::Evidence;
use nomos_workspace::Workspace;

/// Drives `result.plan`, if there is one, through `Preview -> Stage -> Validate -> Commit`
/// against `workspace`, submitting under `evidence`.
///
/// # Errors
///
/// Returns whatever [`nomos_corrections::CorrectionPlan::Stage`],
/// [`nomos_corrections::StagedPlan::Validate`] or
/// [`nomos_corrections::ValidatedPlan::Commit`] refuses with, unchanged: a
/// [`CorrectionError::StaleCandidate`] if the plan's declared prior content no longer
/// matches `workspace`, or a [`CorrectionError::Moved`] if `workspace` advanced between two
/// of the three checked steps. Either refusal is returned before `workspace` is touched --
/// `Commit` is the only one of the four steps able to mutate it, and it re-checks
/// `workspace` has not moved immediately before applying anything.
pub fn Run_Validated_Correction(
    result: &WorkResult, workspace: &mut Workspace, evidence: Evidence,
) -> Result<ValidatedCorrectionOutcome, CorrectionError>
{
    let Some(plan) = &result.plan
    else
    {
        return Ok(ValidatedCorrectionOutcome::NoPlan);
    };

    let preview = plan.Preview();
    let staged = plan.Stage(workspace)?;
    let validated = staged.Validate(workspace)?;
    let committed = validated.Commit(workspace, evidence)?;

    return Ok(ValidatedCorrectionOutcome::Committed { preview, committed });
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_agent_contracts::{PortionSubstantiation, Substantiation};
    use nomos_contracts::{ConfigurationId, Digest128, EvidenceClass, ProviderId};
    use nomos_corrections::{ChangeSet, CorrectionCandidate, CorrectionClass, CorrectionPlan, Edit};
    use nomos_model::Content_Digest;
    use nomos_workspace::{BuildVariant, ChangeSource, WorkspaceChangeSet};

    const CONFIGURATION_SEED_BYTE: u8 = 0x91;

    #[test]
    fn Test_A_Sound_Plan_Should_Commit_Against_The_Real_Workspace()
    {
        let mut workspace = Real_Workspace();
        let starting = workspace.Id();
        let result = Result_Proposing(Rewrite { before: "old", after: "new" });

        let outcome = Run_Validated_Correction(&result, &mut workspace, Agent_Judged()).expect("a sound plan commits cleanly");

        match outcome
        {
            ValidatedCorrectionOutcome::Committed { preview, committed } =>
            {
                assert!(!preview.Rendered().is_empty(), "a real edit renders a real preview");
                assert_ne!(committed.After(), starting, "a real commit must have actually advanced the workspace");
            }
            ValidatedCorrectionOutcome::NoPlan => panic!("the result carried a plan"),
        }
        assert_eq!(
            workspace.Content_Of("a.rs"),
            Some(Content_Digest(b"new")),
            "the committed edit must be visible on the workspace this call was given"
        );
    }

    /// The measured guarantee `done_when` names directly: a refusal leaves the tree
    /// untouched. `Stage` catches this one -- the plan's declared prior content does not
    /// match what `workspace` actually holds -- before `Commit` is ever reached.
    #[test]
    fn Test_A_Refused_Plan_Should_Leave_The_Workspace_Untouched()
    {
        let mut workspace = Real_Workspace();
        let starting = workspace.Id();
        let result = Result_Proposing(Rewrite { before: "not what is there", after: "new" });

        let refusal = Run_Validated_Correction(&result, &mut workspace, Agent_Judged()).expect_err("the declared prior content is wrong");

        assert!(matches!(refusal, CorrectionError::StaleCandidate { .. }), "{refusal:?}");
        assert_eq!(workspace.Id(), starting, "a refused plan must not advance the workspace");
        assert_eq!(
            workspace.Content_Of("a.rs"),
            Some(Content_Digest(b"old")),
            "a refused plan must not change what the workspace holds"
        );
    }

    /// `OD-CONTRACTS-003`'s case: a judgment-only result proposes nothing, and that is not
    /// a refusal -- there was nothing to stage, validate or commit.
    #[test]
    fn Test_A_Judgment_Only_Result_Should_Report_No_Plan_And_Touch_Nothing()
    {
        let mut workspace = Real_Workspace();
        let starting = workspace.Id();
        let result = Judgment_Only_Result();

        let outcome = Run_Validated_Correction(&result, &mut workspace, Agent_Judged()).expect("no plan is not a refusal");

        assert_eq!(outcome, ValidatedCorrectionOutcome::NoPlan);
        assert_eq!(workspace.Id(), starting);
    }

    /// The declared portions are all grounded because this module is their producer and
    /// built them itself, so the absent ones mean this task proposed none rather than that
    /// a dispatch could not ground them (`OD-EXECUTOR-011`).
    fn Judgment_Only_Result() -> WorkResult
    {
        return WorkResult {
            plan: None,
            claims: Vec::new(),
            tests: Vec::new(),
            requested_verification: None,
            assumptions: Vec::new(),
            unresolved_questions: vec!["does this warrant a follow-up correction?".to_owned()],
            substantiation: Substantiation {
                plan: PortionSubstantiation::Substantiated,
                claims: PortionSubstantiation::Substantiated,
                tests: PortionSubstantiation::Substantiated,
                requested_verification: PortionSubstantiation::Substantiated,
                assumptions: PortionSubstantiation::Substantiated,
                unresolved_questions: PortionSubstantiation::Substantiated,
            },
        };
    }

    /// A real workspace, built the way a real consumer would, with `a.rs` present holding
    /// `"old"` -- the same shape `nomos-corrections`' own public-API test uses.
    fn Real_Workspace() -> Workspace
    {
        let variant = BuildVariant::New("x86_64-unknown-none", "test", "fixed", Vec::<String>::new());
        let configuration = ConfigurationId::From_Digest(Digest128::From_Bytes([CONFIGURATION_SEED_BYTE; Digest128::BYTE_LENGTH]));
        let mut workspace = Workspace::Empty(variant, configuration);

        let initial = WorkspaceChangeSet::From(ChangeSource::GitCheckout).Present("a.rs", "old");
        workspace.Apply(&initial).expect("Workspace::Empty holds no entry for a.rs, so this Present inserts rather than contradicts");

        return workspace;
    }

    fn Agent_Judged() -> Evidence
    {
        return Evidence {
            class: EvidenceClass::AgentJudged,
            producer: ProviderId::New("nomos-agent-orchestration"),
            supporting: Vec::new(),
        };
    }

    fn Result_Proposing(requested: Rewrite<'_>) -> WorkResult
    {
        let edit = Edit::New("a.rs", Some(requested.before.to_owned()), Some(requested.after.to_owned()));
        let candidate = CorrectionCandidate::New("rewrite a.rs", ChangeSet::Empty().With(edit), CorrectionClass::Agent, vec![]);
        let plan = CorrectionPlan::New(vec![candidate]).expect("one candidate is a valid plan");

        return WorkResult {
            plan: Some(plan),
            claims: Vec::new(),
            tests: Vec::new(),
            requested_verification: None,
            assumptions: Vec::new(),
            unresolved_questions: Vec::new(),
            // Grounded for the reason `Judgment_Only_Result`'s own declaration is: this
            // module built the plan it proposes, so the portions it left absent are ones it
            // had nothing to say about rather than ones no dispatch could have grounded.
            substantiation: Substantiation {
                plan: PortionSubstantiation::Substantiated,
                claims: PortionSubstantiation::Substantiated,
                tests: PortionSubstantiation::Substantiated,
                requested_verification: PortionSubstantiation::Substantiated,
                assumptions: PortionSubstantiation::Substantiated,
                unresolved_questions: PortionSubstantiation::Substantiated,
            },
        };
    }

    /// The two contents one `Result_Proposing` call names: what `a.rs` holds now, and what
    /// the plan proposes it should hold instead.
    ///
    /// Named fields rather than two adjacent `&str` parameters, so a caller cannot hand the
    /// pair over in the wrong order -- `a.rs`'s prior content and its replacement are both
    /// strings, and nothing in a positional call site tells them apart.
    struct Rewrite<'a>
    {
        before: &'a str,
        after: &'a str,
    }
}
