//! Why a dispatch to Claude Code did not produce an [`crate::AgentExecutionOutcome`].

/// Why `Execute` could not answer.
///
/// Two ways, kept apart the same reason `nomos_lang_rust_cargo::MetadataError` collapses
/// every launcher failure into one variant and every parse failure into another rather
/// than a single opaque string: a caller deciding whether to retry needs to know which
/// side of the process boundary the failure was on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentExecutionError
{
    /// The process could not be started, exited non-zero, or was killed for timing out
    /// or stalling. Carries the reason as reported by the launcher or the process's own
    /// stderr.
    Unavailable(String),
    /// The process ran and exited cleanly, but its stdout was not the JSON document
    /// `--output-format json` promises, or was missing a field this reader requires.
    Unparseable(String),
}

impl core::fmt::Display for AgentExecutionError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Unavailable(reason) | Self::Unparseable(reason) => write!(formatter, "{reason}"),
        };
    }
}

impl AgentExecutionError
{
    /// The engine's own reason for producing no answer, as this workspace's.
    ///
    /// The two sides draw the same distinction and always have — it moved down
    /// with the dispatch. The wildcard is not laziness: the engine's error is
    /// `#[non_exhaustive]`, so a variant added down there arrives here as
    /// something this workspace has not yet decided about, and the honest
    /// reading of "we do not know what this is" is that no answer was produced.
    #[must_use]
    pub fn From_Engine(error: xvpe_agent_execution::AgentExecutionError) -> Self
    {
        return match error
        {
            xvpe_agent_execution::AgentExecutionError::Unparseable(reason) =>
            {
                Self::Unparseable(reason)
            }
            other => Self::Unavailable(other.to_string()),
        };
    }
}
