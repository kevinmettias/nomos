//! [`TypesFailure`], why the helper gave no answer for a module.

use nomos_contracts::Applicability;

/// Why the helper gave no answer for one module, sorted by what a person does about it -- and never
/// the same thing as an answer with no value in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypesFailure
{
    /// `go` could not be started: no Go toolchain on this host, or one that cannot find its own
    /// root.
    GoUnavailable,
    /// The helper's source could not be written where `go` can run it: no temporary directory the
    /// environment names, or one that refuses the write.
    HelperUnwritable,
    /// The helper was started and did not finish within its bound.
    DidNotFinish,
    /// The helper finished and failed: it did not build, or `go list` could not answer for the
    /// module.
    Failed,
    /// The helper finished and printed something that is not the answer this provider reads.
    Unreadable,
}

impl TypesFailure
{
    /// How a rule reports this failure. A toolchain that cannot run, a helper that cannot be put
    /// where it runs, or a run that does not end is `ProviderUnavailable`: the provider is
    /// installed and could not answer. A run that ended and failed is `AnalysisFailed`: the tool
    /// ran, and what it was given did not load.
    #[must_use]
    pub const fn Applicability(self) -> Applicability
    {
        return match self
        {
            Self::GoUnavailable | Self::HelperUnwritable | Self::DidNotFinish => Applicability::ProviderUnavailable,
            Self::Failed | Self::Unreadable => Applicability::AnalysisFailed,
        };
    }
}
