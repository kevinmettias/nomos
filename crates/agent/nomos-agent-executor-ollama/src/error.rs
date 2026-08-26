//! Why a dispatch to a local Ollama model did not produce an [`crate::AgentExecutionOutcome`].

/// Why `Execute` could not answer.
///
/// One variant, unlike `nomos_agent_executor_claude_code::AgentExecutionError`'s two: this
/// backend's stdout is plain text, not a JSON document with fields that can be individually
/// absent or malformed, so there is no distinct "ran cleanly but could not be parsed" case
/// to keep apart from "did not run cleanly." A process that exits zero and produces *some*
/// stdout is a valid response by construction; there is nothing further to fail to parse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentExecutionError
{
    /// The isolated working directory could not be created, the process could not be
    /// started, exited non-zero, or was killed for timing out or stalling. Carries the
    /// reason as reported by the launcher or the process's own stderr -- including, when
    /// the cause is an unreachable `ollama serve` daemon, `ollama run`'s own real failure
    /// text (verified directly: `ollama run` does not fail fast on an unreachable daemon,
    /// it attempts to launch `ollama app` to start one, which crashes on this machine's own
    /// Windows tray integration and times out after roughly a minute -- a real, distinct,
    /// if slow and noisy, failure this variant's string still names honestly rather than
    /// hiding behind a generic message).
    Unavailable(String),
}

impl core::fmt::Display for AgentExecutionError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Unavailable(reason) => write!(formatter, "{reason}"),
        };
    }
}
