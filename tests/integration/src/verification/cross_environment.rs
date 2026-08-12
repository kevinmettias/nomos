//! What a scope claim owes beyond one process.

use nomos_contracts::ReproducibilityScope;

/// What a scope claim owes beyond one process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossEnvironment
{
    /// `SingleRun`. Nothing outside this process was promised.
    Nothing,
    /// `CrossRun`. A second process must reach the same bytes.
    SecondProcess,
    /// `CrossPlatform` or `CrossBinary`. A second process, and agreement with a digest
    /// captured on another platform or another build — which in a repository is a
    /// committed constant, because the other platform is not here to ask.
    SecondProcessAndGolden,
}

/// The obligation a scope declaration carries beyond one process.
///
/// A function of the declaration alone, so a caller cannot discharge the obligations of a
/// weaker scope than the one its domain declared.
#[must_use]
pub const fn Cross_Environment_Owed(scope: ReproducibilityScope) -> CrossEnvironment
{
    return match scope
    {
        ReproducibilityScope::SingleRun => CrossEnvironment::Nothing,
        ReproducibilityScope::CrossRun => CrossEnvironment::SecondProcess,
        ReproducibilityScope::CrossPlatform | ReproducibilityScope::CrossBinary =>
        {
            CrossEnvironment::SecondProcessAndGolden
        }
    };
}
