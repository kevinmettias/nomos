//! Which real backend a dispatch reaches.

/// Which real backend [`crate::Run_Agent_Execute`] or [`crate::Run_Agent_Judgment`]
/// dispatches through. Moved here from `nomos_cli::agent::Backend` unchanged in shape:
/// `ClaudeCode` names this workspace's one real `AgentExecutor`
/// (`nomos-agent-executor-claude-code`), `Ollama` its one real `ModelBackend`
/// (`nomos-model-backend-ollama`) dispatched through the same `Execute_Task<Launcher:
/// ProcessLauncher>` shape, not a second `AgentExecutor` -- `OD-PACKAGE-013`'s own
/// finding. A caller-facing surface (a CLI flag, a wire field) still names which one a
/// person or a request chose; this type is only the resolved choice both hosts now share,
/// not a second, host-local copy of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend
{
    ClaudeCode,
    Ollama,
}
