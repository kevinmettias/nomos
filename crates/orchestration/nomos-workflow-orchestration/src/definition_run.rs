//! What running a `WorkflowDefinition` produced, reported in full.

mod branch_choice;
mod dispatch_group;
mod node_disposition;
mod node_report;

pub use branch_choice::BranchChoice;
pub use dispatch_group::DispatchGroup;
pub use node_disposition::NodeDisposition;
pub use node_report::NodeReport;

use crate::{StepAttempt, StepCompensation, WorkflowOutcome};

/// Everything one definition run produced: the outcome the sequential runner already
/// reports, plus the topology that outcome alone cannot carry.
///
/// A third report type beside [`WorkflowOutcome`] and [`crate::WorkflowRun`], for the same
/// reason there was a second: `WorkflowOutcome`'s three variants are matched field by
/// field, with no wildcard arm, by `nomos_api::workflow::WorkflowRunResponse::From` and by
/// `nomos_cli::workflow::report`, and `DispatchError`'s three are matched the same way by
/// `nomos_api::workflow::DispatchErrorResponse::From`. Three exhaustive matches in two
/// host crates this item may not edit, measured rather than assumed. A branch arm or a
/// skipped node expressed as a fourth `WorkflowOutcome` variant would break all three;
/// carried beside it, nothing that compiles today stops compiling.
///
/// [`Self::attempts`] differs from [`crate::WorkflowRun::attempts`] in one way that
/// matters and is deliberate. That field is in the order the dispatches happened, because
/// a sequential run's dispatch order *is* its declared order. Here they are ordered by
/// node and then by attempt, because a group's members have no order between them and
/// reporting the visit order would have leaked it to a caller -- which is the one thing
/// the determinism guarantee promises it will not do.
#[derive(Clone, Debug, PartialEq)]
pub struct DefinitionRun
{
    /// What the run produced, in exactly the shape [`crate::Run`] already answers with.
    ///
    /// [`WorkflowOutcome::Refused`] never appears here. An incoherent step declaration is
    /// refused by `WorkflowDefinition::Publish`, so a definition that can be run at all is
    /// one whose every declaration was already checked -- the refusal moved earlier rather
    /// than away. The `index` a [`WorkflowOutcome::Failed`] carries is the failing node's
    /// position in the definition, and `completed` holds the outcomes of the step nodes
    /// that ran before it; unlike a sequential run, those two are not the same count,
    /// because a definition holds branch and join nodes that dispatch nothing and skipped
    /// nodes that never ran.
    pub outcome: WorkflowOutcome,
    /// What became of every node that settled, in the definition's declared order.
    pub nodes: Vec<NodeReport>,
    /// The groups this run visited, in the order it visited them.
    pub groups: Vec<DispatchGroup>,
    /// Which arm each branch chose, and why, in the definition's declared order.
    pub choices: Vec<BranchChoice>,
    /// Every attempt at every node that dispatched, ordered by node and then by attempt.
    pub attempts: Vec<StepAttempt>,
    /// What compensating the dispatched nodes of a failed run did, in the reverse of the
    /// definition's declared order.
    ///
    /// Reverse declared order rather than reverse completion order, which the sequential
    /// runner uses and which is the same thing there. Here they differ: a group's members
    /// complete in whatever order they were visited, so unwinding by that order would make
    /// the visit order observable again. A node [`NodeDisposition::Served`] from an
    /// earlier node's result is never compensated -- its body never ran, so there is
    /// nothing of its own to undo, and undoing the node it was served from is that node's
    /// own entry.
    pub compensations: Vec<StepCompensation>,
}
