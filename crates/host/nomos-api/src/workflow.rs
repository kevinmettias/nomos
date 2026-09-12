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
//! second, divergent projection of the same three outcome types.
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

use crate::response::GateRunResponse;
use crate::{check, composition, correction, sources};
use nomos_gate_orchestration::Fresh_Run_Id;
use nomos_platform::Clock;
use nomos_composer_std::{CLOCK, FILE_SYSTEM, LAUNCHER};
use nomos_workflow_orchestration::{Body, CheckBody, CorrectionBody, DispatchError, GateBody, Platform, Run, StepOutcome, WorkflowOutcome, WorkflowStepPlan};
use serde::Serialize;

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

    let platform = Platform { launcher: &LAUNCHER, filesystem: &FILE_SYSTEM };
    let run = Fresh_Run_Id(CLOCK.Now());
    let outcome = Run(std::slice::from_ref(&walked_plan), &platform, &composition::Host_Variant(), run);

    return WorkflowRunResponse::From(outcome);
}

/// `body`, with a `Check`, `Correction` or `Gate` root re-walked for real -- discarding
/// whatever `sources` the caller's own value carried, the identical "always re-walk rather
/// than trust a caller-supplied placeholder" contract `nomos-cli`'s own `workflow.rs` keeps
/// between parsing a command and dispatching it. `ClaudeCode` and `Ollama` pass through
/// unchanged: neither names a root this crate could walk.
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
                Err(WorkflowRunResponse::Completed { completed: vec![StepOutcomeResponse::Check(self::check::CheckResponse::NoSource)] })
            }
            Some(sources) => Ok(Body::Check(CheckBody::New(check.root, sources, check.selected))),
        },
        Body::Correction(correction) => match sources::Walked_Sources(&correction.root)
        {
            None => Err(WorkflowRunResponse::UnreadableRoot { reason: format!("`{}` is not a directory", correction.root.display()) }),
            Some(sources) => Ok(Body::Correction(CorrectionBody::New(correction.root, sources, correction.commit))),
        },
        Body::Gate(gate) => match sources::Walked_Sources(&gate.command.root)
        {
            None => Err(WorkflowRunResponse::UnreadableRoot { reason: format!("`{}` is not a directory", gate.command.root.display()) }),
            Some(sources) => Ok(Body::Gate(GateBody::New(sources, gate.command))),
        },
        other @ (Body::ClaudeCode(_) | Body::Ollama(_)) => Ok(other),
    };
}

/// A serializable twin of [`nomos_workflow_orchestration::WorkflowOutcome`].
///
/// A twin rather than a re-export because the type it mirrors does not derive `Serialize`,
/// for the reason [`crate::response`]'s own doc gives. `UnreadableRoot` is not one of
/// [`WorkflowOutcome`]'s own three variants: it is this handler's own composition-root
/// answer for a named root that is not a directory, the identical case `nomos-cli`'s own
/// `workflow.rs` reports as a bare exit code, outside `Rendered`'s own match, rather than
/// inventing a fourth case inside `WorkflowOutcome` itself for a question that function
/// never asks.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum WorkflowRunResponse
{
    UnreadableRoot
    {
        reason: String,
    },
    Completed
    {
        completed: Vec<StepOutcomeResponse>,
    },
    Refused
    {
        completed: Vec<StepOutcomeResponse>, index: usize,
    },
    Failed
    {
        completed: Vec<StepOutcomeResponse>, index: usize, error: DispatchErrorResponse,
    },
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
                Self::Failed { completed: Completed_Response(completed), index, error: DispatchErrorResponse::From(error) }
            }
        };
    }
}

fn Completed_Response(completed: Vec<StepOutcome>) -> Vec<StepOutcomeResponse>
{
    return completed.into_iter().map(StepOutcomeResponse::From).collect();
}

/// A serializable twin of [`nomos_workflow_orchestration::StepOutcome`].
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "backend")]
pub enum StepOutcomeResponse
{
    ClaudeCode(AgentExecutionOutcomeResponse),
    Ollama(OllamaExecutionOutcomeResponse),
    Check(check::CheckResponse),
    Correction(correction::CorrectionResponse),
    Gate(GateRunResponse),
}

impl StepOutcomeResponse
{
    fn From(outcome: StepOutcome) -> Self
    {
        return match outcome
        {
            StepOutcome::ClaudeCode(outcome) => Self::ClaudeCode(AgentExecutionOutcomeResponse::From(outcome)),
            StepOutcome::Ollama(outcome) => Self::Ollama(OllamaExecutionOutcomeResponse::From(outcome)),
            StepOutcome::Check(outcome) => Self::Check(check::CheckResponse::From(outcome)),
            StepOutcome::Correction(outcome) => Self::Correction(correction::CorrectionResponse::From(outcome)),
            StepOutcome::Gate(result) => Self::Gate(GateRunResponse::From(result)),
        };
    }
}

/// A serializable twin of [`nomos_workflow_orchestration::DispatchError`].
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "backend")]
pub enum DispatchErrorResponse
{
    ClaudeCode(AgentExecutionErrorResponse),
    Ollama(OllamaExecutionErrorResponse),
    Gate(GateRunResponse),
}

impl DispatchErrorResponse
{
    fn From(error: DispatchError) -> Self
    {
        return match error
        {
            DispatchError::ClaudeCode(error) => Self::ClaudeCode(AgentExecutionErrorResponse::From(error)),
            DispatchError::Ollama(error) => Self::Ollama(OllamaExecutionErrorResponse::From(error)),
            DispatchError::Gate(result) => Self::Gate(GateRunResponse::From(result)),
        };
    }
}

/// A serializable twin of [`nomos_agent_executor_claude_code::AgentExecutionOutcome`].
///
/// `assumptions`/`unresolved_questions` are [`nomos_agent_contracts::WorkResult`]'s own two
/// fields this executor can honestly populate -- `OD-EXECUTOR-008`'s decision. `plan`,
/// `claims`, `tests` and `requested_verification` are not projected here because they are
/// always structurally absent for this executor, never because a wire caller could not use
/// them if they existed.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentExecutionOutcomeResponse
{
    pub assumptions: Vec<String>,
    pub unresolved_questions: Vec<String>,
    pub denied_tool_uses: Vec<String>,
    pub is_error: bool,
    pub cost_usd: f64,
    pub duration_ms: u64,
}

impl AgentExecutionOutcomeResponse
{
    fn From(outcome: nomos_agent_executor_claude_code::AgentExecutionOutcome) -> Self
    {
        return Self {
            assumptions: outcome.result.assumptions,
            unresolved_questions: outcome.result.unresolved_questions,
            denied_tool_uses: outcome.denied_tool_uses,
            is_error: outcome.is_error,
            cost_usd: crate::agent::Dollars_Of(outcome.cost),
            duration_ms: outcome.duration_ms,
        };
    }
}

/// A serializable twin of [`nomos_agent_executor_claude_code::AgentExecutionError`].
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum AgentExecutionErrorResponse
{
    Unavailable
    {
        reason: String,
    },
    Unparseable
    {
        reason: String,
    },
    /// A path the envelope forbade changing was changed. Carries that path, not a
    /// `reason`: the wire shape says which field of the envelope was violated rather
    /// than flattening it into prose a caller would have to parse back.
    ProhibitedChange
    {
        path: String,
    },
    /// The envelope declared capabilities no tool grant exists for, so nothing ran.
    UnsupportedTools
    {
        capabilities: String,
    },
    /// Paths to protect were declared against a root that does not say which tree.
    UnresolvableRoot
    {
        root: String,
    },
}

impl AgentExecutionErrorResponse
{
    fn From(error: nomos_agent_executor_claude_code::AgentExecutionError) -> Self
    {
        return match error
        {
            nomos_agent_executor_claude_code::AgentExecutionError::Unavailable(reason) => Self::Unavailable { reason },
            nomos_agent_executor_claude_code::AgentExecutionError::Unparseable(reason) => Self::Unparseable { reason },
            nomos_agent_executor_claude_code::AgentExecutionError::ProhibitedChange(path) => Self::ProhibitedChange { path },
            nomos_agent_executor_claude_code::AgentExecutionError::UnsupportedTools(capabilities) => Self::UnsupportedTools { capabilities },
            nomos_agent_executor_claude_code::AgentExecutionError::UnresolvableRoot(root) => Self::UnresolvableRoot { root },
        };
    }
}

/// A serializable twin of [`nomos_model_backend_ollama::AgentExecutionOutcome`].
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OllamaExecutionOutcomeResponse
{
    pub response: String,
}

impl OllamaExecutionOutcomeResponse
{
    fn From(outcome: nomos_model_backend_ollama::AgentExecutionOutcome) -> Self
    {
        return Self { response: outcome.response };
    }
}

/// A serializable twin of [`nomos_model_backend_ollama::AgentExecutionError`].
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum OllamaExecutionErrorResponse
{
    Unavailable
    {
        reason: String,
    },
    /// The envelope declared capabilities no tool grant exists for, so nothing ran. Spelled
    /// the same as its sibling's over the wire, because a caller meets one refusal for one
    /// reason and should not have to learn which backend phrased it.
    UnsupportedTools
    {
        capabilities: String,
    },
}

impl OllamaExecutionErrorResponse
{
    fn From(error: nomos_model_backend_ollama::AgentExecutionError) -> Self
    {
        return match error
        {
            nomos_model_backend_ollama::AgentExecutionError::Unavailable(reason) => Self::Unavailable { reason },
            nomos_model_backend_ollama::AgentExecutionError::UnsupportedTools(capabilities) => Self::UnsupportedTools { capabilities },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{
        Cacheability, CancellationBehavior, Compensation, DeterminismStrength, EvidenceClass, ReproducibilityScope, RetryPolicy, SchemaId, Timeout,
        TraceEquivalence, WorkflowStep,
    };
    use std::path::PathBuf;

    /// The one `WorkflowStep` declaration every test below builds -- fixed and always
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

    /// A real run over a fixture tree with a real blocking phantom claim reaches a real
    /// `Completed` outcome carrying a `Check` step's own `Judged` response -- proving this
    /// crate, not `nomos-cli`, can run a real one-step workflow.
    #[test]
    fn Test_Handle_Workflow_Run_Should_Complete_A_Real_Check_Step()
    {
        let root = std::env::temp_dir().join("nomos-api-workflow-check-step");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a fresh directory");
        std::fs::write(
            root.join("a.rs"),
            "/// A list.\n/// Mirrored by `Test_Api_Workflow_Ghost`.\npub const TABLES: &[&str] = &[];\n",
        )
        .expect("writes a fixture whose stale mirror is one real blocking finding");
        let plan = WorkflowStepPlan { declaration: Coherent_Declaration(), body: Body::Check(CheckBody::New(root.clone(), Vec::new(), Vec::new())) };

        let response = Handle_Workflow_Run(&plan);

        let _ignored = std::fs::remove_dir_all(&root);
        match &response
        {
            WorkflowRunResponse::Completed { completed } => match completed.as_slice()
            {
                [StepOutcomeResponse::Check(check::CheckResponse::Judged { findings, .. })] =>
                {
                    assert!(findings.iter().any(|finding| return finding.summary.contains("Test_Api_Workflow_Ghost")), "{findings:?}");
                }
                other => panic!("expected one Check(Judged) step, got {other:?}"),
            },
            other => panic!("expected Completed, got {other:?}"),
        }
    }

    /// A `Check` step over a root with no source is completed with an honest `NoSource`
    /// step, not a false-clean `Judged` with zero findings -- the guard this handler adds
    /// to match `nomos-cli`'s own `workflow.rs`.
    #[test]
    fn Test_Handle_Workflow_Run_Should_Report_No_Source_For_An_Empty_Check_Step()
    {
        let root = std::env::temp_dir().join("nomos-api-workflow-check-step-empty");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates an empty directory");
        let plan = WorkflowStepPlan { declaration: Coherent_Declaration(), body: Body::Check(CheckBody::New(root.clone(), Vec::new(), Vec::new())) };

        let response = Handle_Workflow_Run(&plan);

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
        let plan = WorkflowStepPlan { declaration: Coherent_Declaration(), body: Body::Check(CheckBody::New(root, Vec::new(), Vec::new())) };

        let response = Handle_Workflow_Run(&plan);

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
        let root = std::env::temp_dir().join("nomos-api-workflow-correction-step-commit");
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a directory");
        std::fs::write(root.join("a.rs"), PHANTOM_FIXTURE).expect("writable");
        let plan =
            WorkflowStepPlan { declaration: Coherent_Declaration(), body: Body::Correction(CorrectionBody::New(root.clone(), Vec::new(), true)) };

        let response = Handle_Workflow_Run(&plan);
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

        let json = serde_json::to_string(&response).expect("a WorkflowRunResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");

        assert_eq!(
            parsed.get("outcome").expect("a serialized WorkflowRunResponse always has this field"),
            "unreadable_root",
            "{json}"
        );
    }
}
