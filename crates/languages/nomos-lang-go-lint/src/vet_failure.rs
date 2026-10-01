//! [`VetFailure`], why `go vet` gave no answer for a module.

use nomos_contracts::Applicability;

/// Why `go vet` gave no answer for one module, sorted by what a person does about it -- and never
/// the same thing as an answer with no diagnostic in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VetFailure
{
    /// `go` could not be started: no Go toolchain on this host, or one that cannot find its own
    /// root.
    GoUnavailable,
    /// `go vet` was started and did not finish within its bound.
    DidNotFinish,
    /// `go vet` finished and failed: the module's packages did not load or type-check, so no
    /// analyzer ran over them.
    Failed,
    /// `go vet` finished and printed something that is not the JSON this provider reads.
    Unreadable,
}

impl VetFailure
{
    /// How a rule reports this failure. A toolchain that cannot run, or a run that does not end, is
    /// `ProviderUnavailable`: the provider is installed and could not answer. A run that ended and
    /// failed is `AnalysisFailed`: the tool ran, and what it was given did not load.
    #[must_use]
    pub const fn Applicability(self) -> Applicability
    {
        return match self
        {
            Self::GoUnavailable | Self::DidNotFinish => Applicability::ProviderUnavailable,
            Self::Failed | Self::Unreadable => Applicability::AnalysisFailed,
        };
    }
}
