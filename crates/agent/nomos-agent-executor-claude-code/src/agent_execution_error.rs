//! Why a dispatch to Claude Code did not produce an [`crate::AgentExecutionOutcome`].

/// Why `Execute` could not answer.
///
/// Four ways, kept apart the same reason `nomos_lang_rust_cargo::MetadataError` collapses
/// every launcher failure into one variant and every parse failure into another rather
/// than a single opaque string: a caller deciding whether to retry needs to know which
/// side of the process boundary the failure was on.
///
/// The last two are `OD-EXECUTOR-007`'s: a declared envelope constraint this crate
/// enforces is a refusal a caller reads structurally, never a violation buried in an
/// outcome it would have to separately audit.
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
    /// A path `TaskEnvelope::prohibited_changes` names was not the same after the
    /// dispatch as before it. Carries the path that changed.
    ProhibitedChange(String),
    /// `TaskEnvelope::available_tools` named capabilities this crate has no way to grant,
    /// so the dispatch never ran. Carries the capabilities named.
    ///
    /// Refusing rather than dispatching a task whose declared requirement this crate
    /// would silently drop: the same "unknown is not pass" reading `Applicability`'s own
    /// module doc states for a rule's judgment.
    UnsupportedTools(String),
    /// `TaskEnvelope::prohibited_changes` named paths to protect and the `root` they
    /// resolve against is not absolute, so which tree they name is not decidable.
    /// Carries the root as given.
    ///
    /// A relative root would resolve against whatever directory the process happens to
    /// be in, so the paths compared could be a different checkout's — protecting the
    /// wrong tree while reporting success. Refusing is the difference between a caller
    /// that has no root yet and one that silently got the wrong one.
    UnresolvableRoot(String),
}

impl core::fmt::Display for AgentExecutionError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Unavailable(reason) | Self::Unparseable(reason) => write!(formatter, "{reason}"),
            Self::ProhibitedChange(path) =>
            {
                write!(formatter, "{path} is named in prohibited_changes and was changed")
            }
            Self::UnsupportedTools(capabilities) =>
            {
                write!(formatter, "no tool grant exists for {capabilities}")
            }
            Self::UnresolvableRoot(root) =>
            {
                write!(formatter, "prohibited_changes needs an absolute root, and {root:?} is not")
            }
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
