//! Which real backend a dispatch reaches.

/// Which real backend [`crate::Run_Agent_Execute`] or [`crate::Run_Agent_Judgment`]
/// dispatches through. Moved here from `nomos_cli::agent::Backend` unchanged in shape:
/// `ClaudeCode` names this workspace's one real `AgentExecutor`
/// (`nomos-agent-executor-claude-code`), `Ollama` its one real `ModelBackend`
/// (`nomos-model-backend-ollama`) dispatched through the same `Execute_Task<Launcher:
/// ProgramLauncher>` shape, not a second `AgentExecutor` -- `OD-PACKAGE-013`'s own
/// finding. A caller-facing surface (a CLI flag, a wire field) still names which one a
/// person or a request chose; this type is only the resolved choice both hosts now share,
/// not a second, host-local copy of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend
{
    ClaudeCode,
    Ollama,
}

impl Backend
{
    /// The name a person reaches this backend by: `--executor claude-code` for
    /// [`Backend::ClaudeCode`], `--model-backend ollama` for [`Backend::Ollama`].
    ///
    /// Taken from `nomos_cli::agent`'s own flag parser rather than invented here, because
    /// those are the two spellings this workspace already accepts and a family name a caller
    /// cannot type is not a name anything resolves. That parser refuses `--executor ollama`
    /// and `--model-backend claude-code`, so the label alone deliberately does not say which
    /// flag carries it -- `OD-PACKAGE-013` found the two backends are different kinds of
    /// thing, and the flag is what distinguishes the kinds.
    ///
    /// This is what makes [`ModelSelector::BackendFamily`] the one selector
    /// [`crate::Resolve_Profile`] can answer: the family a profile names is a label these two
    /// backends already have.
    ///
    /// [`ModelSelector::BackendFamily`]: nomos_model_package::ModelSelector::BackendFamily
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::ClaudeCode => "claude-code",
            Self::Ollama => "ollama",
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Both backends, so a third one added without a label of its own is a compile error in
    /// [`Label`] rather than a test that silently stops covering it.
    const ALL: [Backend; 2] = [Backend::ClaudeCode, Backend::Ollama];

    #[test]
    fn Test_Labels_Should_Be_The_Spellings_The_Cli_Accepts()
    {
        assert_eq!(Backend::ClaudeCode.Label(), "claude-code");
        assert_eq!(Backend::Ollama.Label(), "ollama");
    }

    /// A family selector answers by label, so two backends sharing one would make
    /// [`crate::Resolve_Profile`] unable to tell them apart.
    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|backend| return backend.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two backends share a label");
    }
}
