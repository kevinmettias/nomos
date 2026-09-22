//! The port an `AgentExecutorPackage` answers through.

use std::path::Path;

use crate::{AgentExecution, DispatchRefusal, TaskEnvelope};

/// What a generic agent path may ask of an `AgentExecutorPackage`, and the whole of it.
///
/// One port per `PackageKind`, and [`crate::ModelBackend`] is the other. `OD-EXECUTOR-005`
/// measured that an agent executor and a model backend are different kinds of thing
/// answering different questions, which is why `--executor` and `--model-backend` are
/// separate flags; a single port spanning both would have had to return one shape, and the
/// two shapes are not interchangeable.
///
/// An implementation holds whatever it needs to run -- a process launcher, a client, a
/// connection -- so the generic path is not generic over a platform in order to dispatch.
/// That binding is a composition root's concern (`OD-HOST-002`), and it is what lets
/// `nomos-agent-orchestration` name this trait instead of the crates that implement it.
///
/// This trait decides nothing about the capability boundary an implementation runs inside.
/// The isolated working directory, the spend ceiling, the wall bound, the allow-list and
/// the schema-validated output are `OD-EXECUTOR-001`'s, measured against each real
/// mechanism and owned by the adapter, and a port that restated them would be governing a
/// boundary by analogy -- the exact failure `OD-EXECUTOR-004` exists to refuse.
pub trait AgentExecutor
{
    /// Runs `task` and reports what the dispatch established.
    ///
    /// `root` is what `task.prohibited_changes`'s repository-relative paths resolve
    /// against. An implementation that is handed paths to protect and no usable root
    /// refuses rather than protecting the wrong tree.
    ///
    /// # Errors
    ///
    /// [`DispatchRefusal`] when no execution was produced at all, carrying the reason the
    /// implementation stated.
    fn Execute(&self, task: &TaskEnvelope, root: &Path) -> Result<AgentExecution, DispatchRefusal>;
}
