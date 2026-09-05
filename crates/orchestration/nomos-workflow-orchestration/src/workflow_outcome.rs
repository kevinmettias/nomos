//! What running a `WorkflowStepPlan` sequence produced.

mod dispatch_error;
mod step_outcome;

pub use dispatch_error::DispatchError;
pub use step_outcome::StepOutcome;

/// What one [`crate::Run`] produced.
#[derive(Clone, Debug, PartialEq)]
pub enum WorkflowOutcome
{
    /// Every step in the plan ran and dispatched successfully, in order. Vacuously true
    /// of an empty plan.
    Completed
    {
        /// Every step's real outcome, in the order the plan declared them.
        completed: Vec<StepOutcome>,
    },
    /// The step at `index` declared itself incoherent and was never dispatched.
    Refused
    {
        /// Every real outcome from the steps that ran before `index`, in order.
        completed: Vec<StepOutcome>,
        /// The position in the plan of the step that was refused.
        index: usize,
    },
    /// The step at `index` was dispatched and its dispatch failed.
    Failed
    {
        /// Every real outcome from the steps that ran before `index`, in order.
        completed: Vec<StepOutcome>,
        /// The position in the plan of the step whose dispatch failed.
        index: usize,
        /// Why the dispatch failed.
        error: DispatchError,
    },
}
