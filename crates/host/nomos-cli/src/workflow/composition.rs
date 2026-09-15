//! The two values only this crate can supply to [`nomos_workflow_orchestration::Run`]: the one
//! step declaration this verb composes, and what this binary was built as.

use nomos_contracts::{
    Cacheability, CancellationBehavior, Compensation, DeterminismStrength, EvidenceClass, ReproducibilityScope, RetryPolicy, SchemaId,
    Timeout, TraceEquivalence, WorkflowStep,
};
use nomos_workspace::BuildVariant;

/// The one `WorkflowStep` declaration this verb ever builds -- fixed and always coherent, so
/// `Is_Coherent` never refuses the one step this command composes. A future increment parsing a
/// real multi-step plan from argv is what would ever construct one that is not.
pub(super) fn Coherent_Declaration() -> WorkflowStep
{
    return WorkflowStep {
        input_schema: SchemaId::New("nomos.workflow.cli.input.v1"),
        output_schema: SchemaId::New("nomos.workflow.cli.output.v1"),
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

/// The build variant this binary was compiled as. A near-duplicate of `correct.rs`'s own
/// `Correction_Variant`, not a shared dependency on it -- that function is private to `correct`,
/// the identical reasoning `correct.rs`'s own doc gives for not sharing `check.rs`'s walk.
pub(super) fn Workflow_Variant() -> BuildVariant
{
    return BuildVariant::New(
        env!("NOMOS_TARGET"),
        env!("NOMOS_PROFILE"),
        env!("NOMOS_TOOLCHAIN"),
        env!("NOMOS_FEATURES").split(',').filter(|feature| return !feature.is_empty()),
    );
}
