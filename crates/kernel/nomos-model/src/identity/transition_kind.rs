//! The ways one declaration becomes another.

use serde::{Deserialize, Serialize};

/// What happened to a thing's identity between two snapshots.
///
/// Recording the *kind* of change is what lets a finding, a suppression and a metric
/// history follow their subject through a rename instead of being orphaned by it. The
/// prototype could not express any of these, so every one of them presented as a
/// deletion followed by an unrelated arrival.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransitionKind
{
    /// The same thing, unchanged.
    ExactContinuity,
    /// Probably the same thing under a new name.
    ProbableRename,
    /// Probably the same thing in a new location.
    ProbableMove,
    /// The same thing with a changed signature.
    SignatureEvolution,
    /// One thing became several.
    SplitInto,
    /// Several things became one.
    MergedFrom,
    /// A thing with this identity existed, went away, and something with the same
    /// identity came back. Not continuity, and it must not be reported as such.
    Recreated,
    /// Produced by a generator from a source that is itself the thing to track.
    GeneratedFrom,
    /// Continuity could not be established either way.
    Unresolved,
}

impl TransitionKind
{
    /// Whether this transition preserves the subject's accumulated history.
    ///
    /// [`TransitionKind::Recreated`] and
    /// [`TransitionKind::Unresolved`] deliberately do not. Carrying a
    /// suppression across a recreation would silence a finding on code nobody has
    /// reviewed, and carrying one across an unresolved link would do so on the strength
    /// of a guess.
    #[must_use]
    pub const fn Is_History_Preserving(self) -> bool
    {
        return matches!(
            self,
            Self::ExactContinuity
                | Self::ProbableRename
                | Self::ProbableMove
                | Self::SignatureEvolution
        );
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// The rule that stops a suppression from surviving into code nobody reviewed.
    #[test]
    fn Test_Recreation_Should_Not_Preserve_History()
    {
        assert!(!TransitionKind::Recreated.Is_History_Preserving());
        assert!(!TransitionKind::Unresolved.Is_History_Preserving());
        assert!(!TransitionKind::SplitInto.Is_History_Preserving());
        assert!(!TransitionKind::MergedFrom.Is_History_Preserving());
    }

    #[test]
    fn Test_Is_History_Preserving_Should_Be_True_For_Renames_And_Moves()
    {
        assert!(TransitionKind::ExactContinuity.Is_History_Preserving());
        assert!(TransitionKind::ProbableRename.Is_History_Preserving());
        assert!(TransitionKind::ProbableMove.Is_History_Preserving());
        assert!(TransitionKind::SignatureEvolution.Is_History_Preserving());
    }
}
