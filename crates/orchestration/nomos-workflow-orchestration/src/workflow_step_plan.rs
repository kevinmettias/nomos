//! One step of a workflow, paired with what actually dispatches it.

use nomos_contracts::WorkflowStep;

use crate::Body;

/// A `WorkflowStep` declaration paired with the `Body` that dispatches it.
///
/// `WorkflowStep` declares no dispatch mechanism of its own (`OD-WORKFLOW-003`), so
/// pairing one with a concrete `Body` is this crate's own decision, not something the
/// contract itself states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowStepPlan
{
    /// What this step promises.
    pub declaration: WorkflowStep,
    /// What actually dispatches it.
    pub body: Body,
}
