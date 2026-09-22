//! [`Handle_Workflow_Run`] and its own [`WorkflowRunResponse`], paired in one file: the
//! response type exists only for this one handler, the same "handler beside its own
//! response" locality [`crate::correction`] and [`crate::check`] both keep.
//!
//! `P62-API-WORKFLOW-SEAM` gives this crate its first Workflow verb.
//! `nomos_workflow_orchestration::Run` had exactly one caller anywhere in this workspace
//! before this file existed -- `nomos-cli`'s own `workflow.rs`, itself kept to composing a
//! single step per invocation -- so this dispatches the identical single-step shape
//! through this crate's own walk and build variant, [`crate::sources::Walked_Sources`] and
//! [`crate::composition::Host_Variant`], rather than sharing either with `nomos-cli`: a
//! walk and a build variant are each a composition-root concern, `OD-HOST-002`'s own
//! division.
//!
//! # What this reuses rather than re-derives
//!
//! A [`Body::Check`] step's own outcome is rendered through [`crate::CheckResponse`], a
//! [`Body::Correction`] step's through [`crate::CorrectionResponse`], and a
//! [`Body::Gate`] step's (or a failing gate's own [`nomos_workflow_orchestration::
//! DispatchError::Gate`]) through [`crate::response::GateRunResponse`] -- the identical
//! twins this crate's own bare Check, Correction and Gate seams already established, not a
//! second, divergent projection of the same three outcome types. Each of those step and
//! error twins lives in its own `workflow/` submodule and is re-exported here, so this
//! module's own file holds the handler, the plan walk and the outcome response alone.
//!
//! # The one honesty guard this crate adds, matching `nomos-cli`'s own
//!
//! `nomos_workflow_orchestration::Dispatch` calls `nomos_check_orchestration::Run` with
//! whatever `sources` a [`Body::Check`] already carries, empty or not, and that crate's own
//! `Run` does not itself special-case an empty slice -- `CheckOutcome::NoSource` is a
//! composition-root classification, not something `Run` derives from empty input.
//! `nomos-cli`'s own `workflow.rs` pre-empts this before ever calling
//! `nomos_workflow_orchestration::Run`, and this handler does the same, over its own real
//! walk. `Body::Correction`'s own `Run_Correction` already reports `NoSourceFound` for an
//! empty walk itself, so there is no second guard to duplicate for it here -- the identical
//! asymmetry `nomos-cli`'s own module doc names. `Body::Gate` gets no such guard either,
//! for the same reason `nomos-cli`'s own `workflow.rs` does not add one: an empty-tree
//! vacuity gap in `nomos-workflow-orchestration`'s own `Dispatched_Gate` is real, but it is
//! that crate's territory to close, not something this seam papers over by inventing a
//! answer that crate does not give.

use crate::{composition, sources};
use nomos_composer_std::{CLOCK, ENVIRONMENT, FILE_SYSTEM, LAUNCHER};
use nomos_gate_orchestration::Fresh_Run_Id;
use nomos_platform::Clock;
use nomos_workflow_orchestration::{Body, CheckBody, CommitIntent, CorrectionBody, GateBody, Platform, Run, StepOutcome, WorkflowOutcome, WorkflowStepPlan};

mod dispatch_error_response;
mod step_outcome_response;
mod workflow_run_response;

pub use dispatch_error_response::DispatchErrorResponse;
pub use step_outcome_response::StepOutcomeResponse;
pub use workflow_run_response::WorkflowRunResponse;

/// Walks whichever of `plan.body`'s root(s) needs walking, dispatches the one step through
/// `nomos_workflow_orchestration::Run`, and hands back a JSON-serializable
/// [`WorkflowRunResponse`].
#[must_use]
pub fn Handle_Workflow_Run(plan: &WorkflowStepPlan) -> WorkflowRunResponse
{
    let body = match Walked_Body(plan.body.clone())
    {
        Ok(body) => body,
        Err(response) => return response,
    };
    let walked_plan = WorkflowStepPlan { declaration: plan.declaration.clone(), body };

    let declared = crate::agent::Shipped_Targets();
    let platform = Platform {
        launcher: &LAUNCHER,
        filesystem: &FILE_SYSTEM,
        environment: &ENVIRONMENT,
        now: CLOCK.Now(),
        declared: &declared,
    };
    let run = Fresh_Run_Id(CLOCK.Now());
    let outcome = Run(std::slice::from_ref(&walked_plan), &platform, &composition::Host_Variant(), run);

    return WorkflowRunResponse::From(outcome);
}

/// `body`, with a `Check`, `Correction` or `Gate` root re-walked for real -- discarding
/// whatever `sources` the caller's own value carried, the identical "always re-walk rather
/// than trust a caller-supplied placeholder" contract `nomos-cli`'s own `workflow.rs` keeps
/// between parsing a command and dispatching it. `Agent` passes through unchanged: it
/// names no root this crate could walk.
///
/// Returns the already-final [`WorkflowRunResponse`] instead of a walked [`Body`] when a
/// named root is not a directory, or when a `Check` body's real walk found no source --
/// the two composition-root-level answers this function gives before
/// `nomos_workflow_orchestration::Run` is ever called.
fn Walked_Body(body: Body) -> Result<Body, WorkflowRunResponse>
{
    return match body
    {
        Body::Check(check) => match sources::Walked_Sources(&check.root)
        {
            None => Err(WorkflowRunResponse::UnreadableRoot { reason: format!("`{}` is not a directory", check.root.display()) }),
            Some(sources) if sources.is_empty() =>
            {
                Err(WorkflowRunResponse::Completed { completed: vec![StepOutcomeResponse::Check(crate::check::CheckResponse::NoSource)] })
            }
            Some(sources) =>
            {
                let walked = CheckBody::New(check.root, sources, check.selected);
                Ok(Body::Check(walked))
            }
        },
        Body::Correction(correction) => match sources::Walked_Sources(&correction.root)
        {
            None => Err(WorkflowRunResponse::UnreadableRoot { reason: format!("`{}` is not a directory", correction.root.display()) }),
            Some(sources) =>
            {
                let walked = CorrectionBody::New(correction.root, sources, correction.commit);
                Ok(Body::Correction(walked))
            }
        },
        Body::Gate(gate) => match sources::Walked_Sources(&gate.command.root)
        {
            None => Err(WorkflowRunResponse::UnreadableRoot { reason: format!("`{}` is not a directory", gate.command.root.display()) }),
            Some(sources) =>
            {
                let walked = GateBody::New(sources, gate.command);
                Ok(Body::Gate(walked))
            }
        },
        other @ Body::Agent(_) => Ok(other),
    };
}

impl WorkflowRunResponse
{
    fn From(outcome: WorkflowOutcome) -> Self
    {
        return match outcome
        {
            WorkflowOutcome::Completed { completed } => Self::Completed { completed: Completed_Response(completed) },
            WorkflowOutcome::Refused { completed, index } => Self::Refused { completed: Completed_Response(completed), index },
            WorkflowOutcome::Failed { completed, index, error } =>
            {
                Self::Failed { completed: Completed_Response(completed), index, error: Error_Response(error) }
            }
        };
    }
}

fn Completed_Response(completed: Vec<StepOutcome>) -> Vec<StepOutcomeResponse>
{
    return completed.into_iter().map(StepOutcomeResponse::From).collect();
}

/// The `Failed` arm's own `error` field, converted for the wire -- the peer of
/// [`Completed_Response`], which converts that same arm's `completed` field.
fn Error_Response(error: nomos_workflow_orchestration::DispatchError) -> DispatchErrorResponse
{
    return DispatchErrorResponse::From(error);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{check, correction};
    use nomos_contracts::{
        Cacheability, CancellationBehavior, Compensation, DeterminismStrength, EvidenceClass, ReproducibilityScope, RetryPolicy, SchemaId, Timeout,
        TraceEquivalence, WorkflowStep,
    };
    use std::path::{Path, PathBuf};

    /// The subject `check-completeness-mirror` reports in the source every fresh check root
    /// here is written with: a stale mirror naming a `Test_*` this crate does not hold, which
    /// is one real blocking finding rather than a fixture-only one.
    const GHOST: &str = "Test_Api_Workflow_Ghost";

    /// A real source file carrying exactly that one blocking stale mirror.
    const BLOCKING_PHANTOM: &str = "/// A list.\n/// Mirrored by `Test_Api_Workflow_Ghost`.\npub const TABLES: &[&str] = &[];\n";

    /// A real run over a fixture tree with a real blocking phantom claim reaches a real
    /// `Completed` outcome carrying a `Check` step's own `Judged` response -- proving this
    /// crate, not `nomos-cli`, can run a real one-step workflow.
    #[test]
    fn Test_Handle_Workflow_Run_Should_Complete_A_Real_Check_Step()
    {
        let root = Fresh_Root("nomos-api-workflow-check-step");
        Write_Source(&root, BLOCKING_PHANTOM);

        let response = Handle_Workflow_Run(&Plan_Over(Check_Body(&root)));

        let _ignored = std::fs::remove_dir_all(&root);
        let judged = Judged_Summaries(&response);

        assert!(judged.iter().any(|summary| return summary.contains(GHOST)), "{judged:?}");
    }

    /// The summaries of every finding the single `Check(Judged)` step of `response` reports,
    /// or a panic naming what `response` held instead.
    fn Judged_Summaries(response: &WorkflowRunResponse) -> Vec<&str>
    {
        let WorkflowRunResponse::Completed { completed } = response
        else
        {
            panic!("expected a completed run, got {response:?}");
        };
        let [StepOutcomeResponse::Check(check::CheckResponse::Judged { findings, .. })] = completed.as_slice()
        else
        {
            panic!("expected one Check(Judged) step, got {completed:?}");
        };

        return findings.iter().map(|finding| return finding.summary.as_str()).collect();
    }

    /// A `Check` step over a root with no source is completed with an honest `NoSource`
    /// step, not a false-clean `Judged` with zero findings -- the guard this handler adds
    /// to match `nomos-cli`'s own `workflow.rs`.
    #[test]
    fn Test_Handle_Workflow_Run_Should_Report_No_Source_For_An_Empty_Check_Step()
    {
        let root = Fresh_Root("nomos-api-workflow-check-step-empty");

        let response = Handle_Workflow_Run(&Plan_Over(Check_Body(&root)));

        let _ignored = std::fs::remove_dir_all(&root);
        assert!(
            matches!(&response, WorkflowRunResponse::Completed { completed } if matches!(completed.as_slice(), [StepOutcomeResponse::Check(check::CheckResponse::NoSource)])),
            "{response:?}"
        );
    }

    /// A root that does not exist is `UnreadableRoot`, the composition-root-level answer
    /// this handler gives before `nomos_workflow_orchestration::Run` is ever called.
    #[test]
    fn Test_Handle_Workflow_Run_Should_Report_Unreadable_Root_For_A_Missing_Directory()
    {
        let root: PathBuf = std::env::temp_dir().join("nomos-api-workflow-missing-root-does-not-exist");
        let _ignored = std::fs::remove_dir_all(&root);

        let response = Handle_Workflow_Run(&Plan_Over(Check_Body(&root)));

        assert!(matches!(response, WorkflowRunResponse::UnreadableRoot { .. }), "{response:?}");
    }

    /// A real run committing a real phantom claim through a `Correction` step reaches a
    /// real `Completed` outcome carrying that step's own `Committed` response.
    #[test]
    fn Test_Handle_Workflow_Run_Should_Commit_A_Real_Correction_Step()
    {
        const PHANTOM_FIXTURE: &str = "/// A list of things this crate owns.\n\
            /// Mirrored by `Test_Nonexistent_Api_Workflow_Check_That_Does_Not_Exist`.\n\
            pub const THINGS: &[&str] = &[\"a\"];\n";
        let root = Fresh_Root("nomos-api-workflow-correction-step-commit");
        Write_Source(&root, PHANTOM_FIXTURE);
        let body = CorrectionBody::New(root.clone(), Vec::new(), CommitIntent::Commit);

        let response = Handle_Workflow_Run(&Plan_Over(Body::Correction(body)));
        let corrected = std::fs::read_to_string(root.join("a.rs")).expect("still readable");

        let _ignored = std::fs::remove_dir_all(&root);
        assert!(
            matches!(&response, WorkflowRunResponse::Completed { completed } if matches!(completed.as_slice(), [StepOutcomeResponse::Correction(correction::CorrectionResponse::Committed { .. })])),
            "{response:?}"
        );
        assert_eq!(corrected, "/// A list of things this crate owns.\npub const THINGS: &[&str] = &[\"a\"];\n");
    }

    /// The whole point of this crate: the response a real run produces is valid JSON, and
    /// its outcome round-trips through `serde_json` under the field name a wire caller
    /// would actually read.
    #[test]
    fn Test_From_Should_Produce_A_Response_That_Round_Trips_As_Json()
    {
        let response = WorkflowRunResponse::UnreadableRoot { reason: "fixture".to_owned() };

        let json = serde_json::to_string(&response).expect("a derived Serialize over owned data has nothing to refuse");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");

        assert_eq!(
            parsed.get("outcome").expect("a serialized WorkflowRunResponse always has this field"),
            "unreadable_root",
            "{json}"
        );
    }

    /// The one `WorkflowStep` declaration every test above builds -- fixed and always
    /// coherent, the identical fixture `nomos-cli`'s own `workflow.rs` tests build.
    fn Coherent_Declaration() -> WorkflowStep
    {
        return WorkflowStep {
            input_schema: SchemaId::New("nomos.workflow.api.test.input.v1"),
            output_schema: SchemaId::New("nomos.workflow.api.test.output.v1"),
            has_side_effects: false,
            idempotent: true,
            retry: RetryPolicy::NoRetry,
            timeout: Timeout::Unbounded,
            cacheability: Cacheability::NotCacheable,
            privileges: Vec::new(),
            cancellation: CancellationBehavior::Uncancellable,
            compensation: Compensation::None,
            determinism_strength: DeterminismStrength::None,
            reproducibility_scope: ReproducibilityScope::SingleRun,
            trace_equivalence: TraceEquivalence::NotApplicable,
            evidence: EvidenceClass::AgentJudged,
        };
    }

    /// A one-step plan over `body`, declared by [`Coherent_Declaration`] -- the only shape
    /// this crate's workflow seam accepts.
    fn Plan_Over(body: Body) -> WorkflowStepPlan
    {
        return WorkflowStepPlan { declaration: Coherent_Declaration(), body };
    }

    /// A `Check` body over `root` whose `sources` and `selected` are both empty, so the
    /// handler walks `root` for real rather than trusting a caller-supplied placeholder.
    fn Check_Body(root: &Path) -> Body
    {
        let body = CheckBody::New(root.to_path_buf(), Vec::new(), Vec::new());
        return Body::Check(body);
    }

    /// A fresh, empty scratch directory under `name`, with anything a previous run left
    /// behind removed first so a rerun starts where a first run did.
    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }

    /// `content` written to `root`'s own `a.rs`, which the caller has already created.
    fn Write_Source(root: &Path, content: &str)
    {
        std::fs::write(root.join("a.rs"), content).expect("the fresh root above was just created");
    }
}
