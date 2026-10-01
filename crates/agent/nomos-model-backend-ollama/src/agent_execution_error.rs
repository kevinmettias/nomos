//! Why a dispatch to a local Ollama model did not produce an [`crate::AgentExecutionOutcome`].

/// Why `Execute` could not answer.
///
/// No `Unparseable`, unlike `nomos_agent_executor_claude_code::AgentExecutionError`: this
/// backend's stdout is plain text, not a JSON document with fields that can be individually
/// absent or malformed, so there is no distinct "ran cleanly but could not be parsed" case
/// to keep apart from "did not run cleanly." A process that exits zero and produces *some*
/// stdout is a valid response by construction; there is nothing further to fail to parse.
///
/// [`Self::UnsupportedTools`] *is* shared with that sibling, and deliberately: it is the
/// envelope's own mechanism rather than either backend's, so a caller declaring a capability
/// meets the same refusal whichever one a dispatch chose.
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
    /// `TaskEnvelope::available_tools` named capabilities this backend has no way to grant,
    /// so the dispatch never ran. Carries the capabilities named.
    ///
    /// `OD-EXECUTOR-007`: a backend that cannot honor a declared need says so rather than
    /// proceeding as if it could. Here the need is not merely unmet but unmeetable — this
    /// backend runs one model turn with no tool-use loop at all, so there is no grant it
    /// could ever make.
    UnsupportedTools(String),
}

impl core::fmt::Display for AgentExecutionError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Unavailable(reason) => write!(formatter, "{reason}"),
            Self::UnsupportedTools(capabilities) =>
            {
                write!(formatter, "no tool grant exists for {capabilities}")
            }
        };
    }
}

impl AgentExecutionError
{
    /// The engine's own reason for producing no answer, as this workspace's.
    ///
    /// The wildcard is not laziness: the engine's error is `#[non_exhaustive]`,
    /// so a variant added down there arrives here as something this workspace
    /// has not yet decided about, and the honest reading of "we do not know what
    /// this is" is that no answer was produced.
    #[must_use]
    pub fn From_Engine(error: xvpe_agent_execution::AgentExecutionError) -> Self
    {
        return Self::Unavailable(error.to_string());
    }
}
