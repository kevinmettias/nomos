//! What running a `WorkflowStepPlan` sequence produced, reported in full.

mod step_attempt;
mod step_compensation;
mod step_timing;

pub use step_attempt::StepAttempt;
pub use step_compensation::StepCompensation;
pub use step_timing::StepTiming;

use crate::WorkflowOutcome;

/// Everything one run produced: the outcome [`crate::Run`] reports, plus what honoring
/// each step's declared retry, timeout and compensation actually did.
///
/// A second type beside [`WorkflowOutcome`] rather than three more fields inside it.
/// `WorkflowOutcome`'s own three variants are matched field by field, with no wildcard, by
/// `nomos_api::workflow::WorkflowRunResponse::From` and `nomos_cli::workflow::report`, so
/// a fourth variant or a fifth field there is a breaking change to two crates
/// `P123-WORKFLOW-RETRY-TIMEOUT-COMPENSATION-RUNTIME` may not edit. Carrying the new
/// report beside the old answer keeps both hosts compiling against exactly the shape they
/// already read, and leaves the richer one to callers that ask for it through
/// [`crate::Run_Unclocked`] or [`crate::Run_With_Clock`].
#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowRun
{
    /// What the run produced -- the same answer [`crate::Run`] returns on its own.
    pub outcome: WorkflowOutcome,
    /// Every attempt at every step that was dispatched, in the order the dispatches
    /// happened.
    ///
    /// Empty for a run whose first step declared itself incoherent, which is the direct
    /// evidence that a refused step's body never dispatched at all: `WF-012`'s own named
    /// failure -- a retryable, non-idempotent, side-effecting step with neither a
    /// deduplication token nor a compensation -- is refused by
    /// `WorkflowStep::Is_Coherent` before any of this list could be written.
    pub attempts: Vec<StepAttempt>,
    /// What compensating the completed steps of a failed run did, in the reverse of the
    /// order those steps ran.
    ///
    /// Empty unless [`Self::outcome`] is [`WorkflowOutcome::Failed`]. A run that
    /// completed has nothing to undo. A run [`WorkflowOutcome::Refused`] is left alone
    /// deliberately: the step that refused never dispatched, so no dispatch of it failed,
    /// and whether the steps before an incoherent declaration should be unwound is a
    /// question `WF-012` does not answer and this crate does not decide for it. The
    /// completed steps of a refused run are reported in `completed` exactly as before,
    /// with their effects standing.
    pub compensations: Vec<StepCompensation>,
}
