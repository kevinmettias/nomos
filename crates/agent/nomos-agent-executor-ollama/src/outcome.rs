//! What a bounded Ollama invocation reported.

/// What one `Execute` call reported.
///
/// `response` is free text and is never evidence of what happened -- `OD-EXECUTOR-003`'s own
/// adversarial measurement found this backend's model narrating a file write and a shell
/// command it structurally could not perform, the identical shape
/// `nomos_agent_executor_claude_code`'s own finding names for Claude Code. There is no
/// `denied_tool_uses` field here the way that crate has one: this backend has no tool-use
/// subsystem at all absent `--experimental` (never passed by [`crate::Execute`]), so there is
/// no structural denial signal to read -- the absence of any tool subsystem *is* the entire
/// boundary, not a signal a caller checks after the fact. There is no `cost_usd` either:
/// inference is local, so there is no per-call dollar figure to report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentExecutionOutcome
{
    pub response: String,
}
