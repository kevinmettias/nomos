//! Which real backend a workflow step's body dispatches through.

use nomos_agent_contracts::TaskEnvelope;

/// Which real backend a workflow step's body dispatches through, and the `TaskEnvelope`
/// it carries.
///
/// Names both of this workspace's real dispatch targets directly — `ClaudeCode`, the one
/// real `AgentExecutor`, and `Ollama`, the one real `ModelBackend` — the same shape
/// `nomos_cli::agent::Backend` already uses for a person's own single call, reused rather
/// than reinvented. Not a trait generic over either: `OD-EXECUTOR-001` and `OD-EXECUTOR-004`
/// both decline a shared dispatch trait ahead of a real need, and `OD-PACKAGE-013` settles
/// `Ollama` as a `ModelBackendPackage` instance dispatched through the same shape rather than
/// a second `AgentExecutor` — this crate does not reach past either restraint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Body
{
    /// Dispatches through `nomos-agent-executor-claude-code`.
    ClaudeCode(TaskEnvelope),
    /// Dispatches through `nomos-model-backend-ollama`.
    Ollama(TaskEnvelope),
}
