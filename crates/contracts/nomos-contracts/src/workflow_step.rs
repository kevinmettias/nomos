//! `WorkflowStep` -- what one step of a workflow promises, independent of any engine
//! that would run it.
//!
//! `OD-WORKFLOW-003`'s admission for the workflow tier's vocabulary: the contract a
//! peer executor -- API-hosted, subscription-agent, human, or recorded-replay, `WF-006`'s
//! own list -- must agree with Nomos about before either side can speak about "a step"
//! at all. That agreement cannot wait for a runtime consumer to exist, because the
//! runtime consumer is exactly what this vocabulary lets a future executor become: the
//! same admission `OD-CONTRACTS-001` already gives `RunId`, `EvidenceClass` and
//! `GateCategory` for the same reason -- a peer that never compiles this crate still has
//! to agree with Nomos about what one of these means.
//!
//! This is deliberately not the execution engine `WF-009` through `WF-012` describe.
//! Nothing here schedules, retries or runs anything; a step only *declares* what an
//! engine, once one exists, would have to honor. See `OD-WORKFLOW-003` for the full
//! boundary and for `nomos-gate-orchestration`'s own first real declaration against
//! this shape, over the one real execution this workspace has today.

mod cacheability;
mod cancellation_behavior;
mod compensation;
mod retry_policy;
mod timeout;

pub use cacheability::Cacheability;
pub use cancellation_behavior::CancellationBehavior;
pub use compensation::Compensation;
pub use retry_policy::RetryPolicy;
pub use timeout::Timeout;

use crate::{Declaration_Is_Coherent, DeterminismStrength, EvidenceClass, ReproducibilityScope, SchemaId, TraceEquivalence};
use serde::{Deserialize, Serialize};

/// What one step of a workflow promises, independent of any engine that would run it.
///
/// Every field answers one of `WF-008`'s eleven declared properties. Three reuse
/// vocabulary this crate already has rather than inventing a parallel one:
/// `determinism_strength`/`reproducibility_scope`/`trace_equivalence` are exactly the
/// triple `crate::determinism::Strategy` already declares for a Rust-typed strategy, and
/// `evidence` is the same [`EvidenceClass`] a [`crate::Finding`] carries -- a step's
/// output is evidence of the same five classes, judged the same way.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowStep
{
    /// The schema of what this step accepts.
    pub input_schema: SchemaId,
    /// The schema of what this step produces.
    pub output_schema: SchemaId,
    /// Whether running this step touches anything outside its own return value.
    pub has_side_effects: bool,
    /// Whether re-running this step with the same inputs is safe without
    /// deduplication -- the same effect, not a second one.
    pub idempotent: bool,
    /// Whether -- and how -- a failed execution may be retried.
    pub retry: RetryPolicy,
    /// How long this step may run before it is no longer waited on.
    pub timeout: Timeout,
    /// Whether a prior result may be substituted for a fresh execution.
    pub cacheability: Cacheability,
    /// What this step needs in order to run, named rather than typed -- the same
    /// "raw, unresolved" choice `cacheability`'s `key_inputs` makes, because no closed
    /// taxonomy of privilege categories has a second real executor to check it against
    /// yet.
    pub privileges: Vec<String>,
    /// What happens when cancellation is requested mid-execution.
    pub cancellation: CancellationBehavior,
    /// Whether this step's own effect can be undone, and by what.
    pub compensation: Compensation,
    /// What kind of sameness this step's determinism claim promises.
    pub determinism_strength: DeterminismStrength,
    /// Across what environment that promise holds.
    pub reproducibility_scope: ReproducibilityScope,
    /// What "the same" means when comparing two of this step's executions.
    pub trace_equivalence: TraceEquivalence,
    /// How this step's own output was come by.
    pub evidence: EvidenceClass,
}

impl WorkflowStep
{
    /// Whether this declaration is internally coherent.
    ///
    /// Two cross-field rules, not a validity check on any one field alone:
    ///
    /// - The determinism triple must itself be coherent, the same rule
    ///   [`crate::determinism::Declaration_Is_Coherent`] already states for any strategy.
    /// - `WF-012`: a step with side effects that is not idempotent must not declare
    ///   [`RetryPolicy::Retry`] unless it either requires a deduplication token or
    ///   declares a [`Compensation`] other than [`Compensation::None`]. Retrying such a
    ///   step blind would repeat a non-idempotent effect with nothing to tell two
    ///   attempts apart -- the exact failure `WF-012`'s own text names.
    #[must_use]
    pub fn Is_Coherent(&self) -> bool
    {
        if !Declaration_Is_Coherent(self.determinism_strength, self.trace_equivalence)
        {
            return false;
        }

        if let RetryPolicy::Retry { deduplication_token_required, .. } = self.retry
        {
            let unsafe_to_repeat = self.has_side_effects && !self.idempotent;
            let retry_is_covered = deduplication_token_required || self.compensation != Compensation::None;

            if unsafe_to_repeat && !retry_is_covered
            {
                return false;
            }
        }

        return true;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::num::NonZeroU32;

    #[test]
    fn Test_Is_Coherent_Should_Accept_A_Step_With_No_Retry()
    {
        assert!(Base().Is_Coherent());
    }

    /// The exact failure `WF-012`'s own text names: a non-idempotent, side-effecting
    /// step that retries with no way to tell two attempts apart.
    #[test]
    fn Test_An_Uncovered_Retry_Of_A_Non_Idempotent_Side_Effecting_Step_Should_Be_Incoherent()
    {
        let mut step = Base();
        step.retry = RetryPolicy::Retry {
            max_attempts: NonZeroU32::new(3).expect("3 is nonzero"),
            deduplication_token_required: false,
        };

        assert!(!step.Is_Coherent());
    }

    #[test]
    fn Test_A_Deduplication_Token_Should_Cover_A_Non_Idempotent_Retry()
    {
        let mut step = Base();
        step.retry = RetryPolicy::Retry {
            max_attempts: NonZeroU32::new(3).expect("3 is nonzero"),
            deduplication_token_required: true,
        };

        assert!(step.Is_Coherent());
    }

    #[test]
    fn Test_A_Compensation_Should_Cover_A_Non_Idempotent_Retry()
    {
        let mut step = Base();
        step.retry = RetryPolicy::Retry {
            max_attempts: NonZeroU32::new(3).expect("3 is nonzero"),
            deduplication_token_required: false,
        };
        step.compensation = Compensation::SelfCompensating;

        assert!(step.Is_Coherent());
    }

    /// An idempotent step has nothing WF-012 protects against: retrying it repeats the
    /// same effect, not a second one, whether or not it has side effects.
    #[test]
    fn Test_An_Idempotent_Step_May_Retry_Freely()
    {
        let mut step = Base();
        step.idempotent = true;
        step.retry = RetryPolicy::Retry {
            max_attempts: NonZeroU32::new(3).expect("3 is nonzero"),
            deduplication_token_required: false,
        };

        assert!(step.Is_Coherent());
    }

    /// A step with no side effects has nothing to repeat, so an uncovered retry is
    /// vacuously safe regardless of the `idempotent` field.
    #[test]
    fn Test_A_Pure_Step_May_Retry_Freely()
    {
        let mut step = Base();
        step.has_side_effects = false;
        step.retry = RetryPolicy::Retry {
            max_attempts: NonZeroU32::new(3).expect("3 is nonzero"),
            deduplication_token_required: false,
        };

        assert!(step.Is_Coherent());
    }

    /// The determinism half of coherence is not this type's own invention -- it defers
    /// entirely to the shared rule every other strategy declaration already answers to.
    #[test]
    fn Test_An_Incoherent_Determinism_Declaration_Should_Make_The_Step_Incoherent()
    {
        let mut step = Base();
        step.determinism_strength = DeterminismStrength::State;
        step.trace_equivalence = TraceEquivalence::NotApplicable;

        assert!(!step.Is_Coherent());
    }

    fn Base() -> WorkflowStep
    {
        return WorkflowStep {
            input_schema: SchemaId::New("nomos.workflow.test.input.v1"),
            output_schema: SchemaId::New("nomos.workflow.test.output.v1"),
            has_side_effects: true,
            idempotent: false,
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
}
