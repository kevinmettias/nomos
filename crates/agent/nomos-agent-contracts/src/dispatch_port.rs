//! Which of the two ports a declared target answers through.

use crate::{AgentExecutor, ModelBackend};

/// Which port a [`crate::DeclaredTarget`] answers through.
///
/// Two variants, and they are the two `PackageKind`s that declare a model selection -- not
/// two vendors. That is the difference this type exists to keep: a generic path matching on
/// this is choosing between "an agent executor answered" and "a model backend answered",
/// which is a distinction `OD-EXECUTOR-005` measured, rather than between Claude Code and
/// Ollama, which is a pair a composition root supplies.
///
/// A third real package kind is a third variant and a compile error at every match, which is
/// what makes adding one a decision. A third *vendor* is neither: it is one more
/// [`crate::DeclaredTarget`] a composition root declares, and nothing here changes.
#[derive(Clone, Copy)]
pub enum DispatchPort<'port>
{
    /// An `AgentExecutorPackage`, answering an [`crate::AgentExecution`].
    Executor(&'port dyn AgentExecutor),
    /// A `ModelBackendPackage`, answering a [`crate::ModelAnswer`].
    Model(&'port dyn ModelBackend),
}
