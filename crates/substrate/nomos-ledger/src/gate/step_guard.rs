//! What a workflow step's `if:` key says about which hosts run it.

/// The guard a step declares, read as a closed subset rather than evaluated.
///
/// GitHub's expression language is not this workspace's, and a reader that guessed at an
/// unimplemented spelling would decide which leg a step belongs to by accident. So this
/// carries exactly two answers it understands and one that says it does not understand,
/// and `OD-GATE-033` makes the third a refusal rather than a default: a guard the executor
/// does not implement must refuse the step rather than guess which leg it selects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepGuard
{
    /// The step declares no `if:` at all, so every leg runs it.
    Unguarded,
    /// `matrix.os == '<host>'`, the one guarded spelling this reader implements.
    Host(String),
    /// An `if:` outside the subset, carried verbatim so a refusal can quote it.
    Outside(String),
}

impl StepGuard
{
    /// Whether `host` is admitted by this guard.
    ///
    /// [`Self::Outside`] answers `false`, and that answer is never the whole story: a step
    /// whose guard is outside the subset is refused rather than reported unavailable, and
    /// the two are different values in [`super::StepExecution`] for the reason
    /// `OD-GATE-001` gives -- "nothing could look" must not read as "nothing was wrong".
    #[must_use]
    pub fn Admits(&self, host: &str) -> bool
    {
        return match self
        {
            Self::Unguarded => true,
            Self::Host(admitted) => admitted == host,
            Self::Outside(_) => false,
        };
    }
}
