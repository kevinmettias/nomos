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
    /// Every backend this build carries, in declaration order.
    ///
    /// Public and outside the test module because [`crate::Declared_Targets`] derives the
    /// declared package set from it: the set of backends is written once, here, so a third
    /// backend cannot reach dispatch without also being declared. It was a test-only constant
    /// while nothing in production needed the set.
    ///
    /// A variant missing from this array is not a compile error; a variant missing a label
    /// already is one, in [`Self::Label`].
    ///
    /// Mirrored by `Test_Every_Variant_Should_Be_Listed`, which compares this array against
    /// the variants named in the test module beside it, in both directions. That sentence
    /// named the test before the test existed: `P122` wrote it, because the claim was the
    /// half already committed.
    pub const ALL: [Self; 2] = [Self::ClaudeCode, Self::Ollama];

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
    use super::Backend as Subject;
    const ALL: [Subject; 2] = Subject::ALL;

    #[test]
    fn Test_Labels_Should_Be_The_Spellings_The_Cli_Accepts()
    {
        assert_eq!(Backend::ClaudeCode.Label(), "claude-code");
        assert_eq!(Backend::Ollama.Label(), "ollama");
    }

    /// Every variant reaches [`Backend::ALL`], which is the claim that constant's own doc
    /// comment makes and which nothing here checked until this test.
    ///
    /// The array is written by hand and sized by hand, so a variant left out of it is not a
    /// compile error and dispatch would simply never reach that backend.
    ///
    /// The mirror is between two independent spellings of the same set: the array, which
    /// production reads, and the variants named below, which this test writes. They must
    /// agree. A third variant makes the `match` non-exhaustive — it carries no wildcard arm
    /// — so the test stops compiling until somebody names the variant here, and naming it
    /// here while leaving `[Self; 2]` alone fails the assertion beneath. A count would not do
    /// that job, because a count is a number somebody has to remember to raise.
    ///
    /// `Test_Labels_Are_Distinct` is not this check, though while the enum has exactly as
    /// many variants as the array has slots the two overlap: omitting one then requires
    /// duplicating another, which collides their labels, and both tests fail. Measured, on a
    /// copy of the array holding `ClaudeCode` twice. What distinctness cannot see is the case
    /// this test exists for — a third variant added while the array keeps two entries that
    /// are still distinct.
    #[test]
    fn Test_Every_Variant_Should_Be_Listed()
    {
        for listed in ALL
        {
            match listed
            {
                Subject::ClaudeCode | Subject::Ollama =>
                {}
            }
        }

        for expected in [Subject::ClaudeCode, Subject::Ollama]
        {
            assert!(
                ALL.contains(&expected),
                "{expected:?} is a Backend variant that Backend::ALL does not list"
            );
        }
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
