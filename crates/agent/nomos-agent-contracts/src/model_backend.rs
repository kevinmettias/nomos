//! The port a `ModelBackendPackage` answers through.

use crate::{DispatchRefusal, ModelAnswer, TaskEnvelope};

/// What a generic agent path may ask of a `ModelBackendPackage`, and the whole of it.
///
/// The sibling of [`crate::AgentExecutor`], and deliberately not the same trait. A model
/// backend answers a prompt; an agent executor runs a bounded, tool-aware turn and reports
/// what its boundary refused and what it spent. `OD-EXECUTOR-005` measured that difference
/// and `OD-PACKAGE-013` named the two package kinds; one trait over both would have to
/// return one shape, and the honest shapes differ -- see [`ModelAnswer`] for what this one
/// deliberately does not carry.
///
/// There is no `root` parameter here, unlike its sibling, because there is nothing for one
/// to resolve against: this port grants no tools, so a declared `prohibited_changes` is
/// refused by the implementation rather than compared around the dispatch.
pub trait ModelBackend
{
    /// Answers `task` and reports what the model said.
    ///
    /// # Errors
    ///
    /// [`DispatchRefusal`] when no answer was produced at all, carrying the reason the
    /// implementation stated -- including an unreachable daemon, which `OD-EXECUTOR-004`
    /// requires be reported as its own real failure rather than folded into silence.
    fn Answer(&self, task: &TaskEnvelope) -> Result<ModelAnswer, DispatchRefusal>;
}
