//! Whether a correction body commits the fix it validated, or stops once it has staged one.

/// Whether a [`crate::CorrectionBody`]'s step commits the fix it validated or stops short of
/// writing it.
///
/// A named pair rather than a `bool`, because a position is not a name: `New(root, sources,
/// true)` does not say what `true` asks for, while [`Self::Commit`] and [`Self::Stage`] do.
/// The two states are the two halves of
/// [`nomos_correction_orchestration::CorrectionOutcome`], which is what a run reports back
/// once it has acted on one of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitIntent
{
    /// Stage and validate the corrected content, and stop before writing it -- `--commit`
    /// absent.
    Stage,
    /// Stage, validate and commit the corrected content, writing it to disk -- `--commit`
    /// given.
    Commit,
}

impl CommitIntent
{
    /// The state `flag` names, at the one boundary where the answer genuinely is a flag:
    /// `nomos_correction_orchestration::CorrectionCommand::commit` and the CLI's own
    /// `--commit` are both booleans, and this is where one becomes a name.
    #[must_use]
    pub fn From_Flag(flag: bool) -> Self
    {
        return if flag { Self::Commit } else { Self::Stage };
    }

    /// Whether this intent commits -- the same boundary read the other way, since
    /// `CorrectionCommand` still takes a `bool`.
    #[must_use]
    pub fn Commits(self) -> bool
    {
        return matches!(self, Self::Commit);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Flag_Should_Name_The_State_The_Flag_Asks_For()
    {
        assert_eq!(CommitIntent::From_Flag(true), CommitIntent::Commit);
        assert_eq!(CommitIntent::From_Flag(false), CommitIntent::Stage);
    }

    #[test]
    fn Test_Commits_Should_Report_Only_The_Committing_State()
    {
        assert!(CommitIntent::Commit.Commits());
        assert!(!CommitIntent::Stage.Commits());
    }
}
