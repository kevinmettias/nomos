//! [`EvaluationFailure`], why `MSBuild` gave no definition set.

use nomos_contracts::Applicability;

/// Why no definition set came back, sorted by what a person can do about it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvaluationFailure
{
    /// The host has no .NET SDK: `dotnet` could not be started, or it started and found no SDK.
    DotnetUnavailable,
    /// `MSBuild` started and did not finish: it timed out, stalled or was terminated.
    DidNotFinish,
    /// `MSBuild` finished and refused: the project is missing or malformed, is not SDK-style, names
    /// no single target framework for the request, or answered in a shape this reader does not
    /// know.
    Refused,
}

impl EvaluationFailure
{
    /// How a rule reading this capability reports a file it could not judge for this reason.
    ///
    /// A missing SDK is `DependencyUnavailable`, which `Applicability` defines as the provider
    /// being present and runnable while "something it needs — an SDK, a toolchain, a license, a
    /// runtime — is not", and keeps apart from `ProviderUnavailable` "because the remedy is
    /// different and the user can act on it": install the SDK. A tool that started and never
    /// answered is `ProviderUnavailable`. A refusal is `AnalysisFailed`: `MSBuild` ran and failed,
    /// and a source file that parses was never the problem.
    #[must_use]
    pub const fn Applicability(self) -> Applicability
    {
        return match self
        {
            Self::DotnetUnavailable => Applicability::DependencyUnavailable,
            Self::DidNotFinish => Applicability::ProviderUnavailable,
            Self::Refused => Applicability::AnalysisFailed,
        };
    }
}
