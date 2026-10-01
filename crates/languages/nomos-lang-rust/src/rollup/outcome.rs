//! What became of one member the rollup tried to read.

use crate::rollup::{APPROXIMATE, READ, UNREACHABLE};
/// How a member's syntax fact was obtained.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome
{
    /// Answered by a provider the caller's requirement admitted at full standing.
    Read,
    /// Answered by a weaker provider than the requirement asked for.
    ///
    /// `OD-CAPABILITY-003`'s third condition. Without a distinct value the scanner's
    /// answer covers the file the parser refused, `Unreachable` drops to zero, and an
    /// index over five parsed members and one pattern-matched one encodes identically to
    /// one over six parsed members.
    Approximate,
    /// No admitted provider had a readable answer for this member.
    ///
    /// Never silently dropped. A rollup that omits the members it could not read reports a
    /// smaller module as though it were a complete one.
    Unreachable,
}

impl Outcome
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Read => READ,
            Self::Approximate => APPROXIMATE,
            Self::Unreachable => UNREACHABLE,
        };
    }

    /// Whether the member contributed entries. `Approximate` did; it says how good they
    /// are, not whether they are there.
    #[must_use]
    pub const fn Is_Answered(self) -> bool
    {
        return matches!(self, Self::Read | Self::Approximate);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Label_Should_Match_The_Shared_Vocabulary()
    {
        assert_eq!(Outcome::Read.Label(), READ);
        assert_eq!(Outcome::Approximate.Label(), APPROXIMATE);
        assert_eq!(Outcome::Unreachable.Label(), UNREACHABLE);
    }

    #[test]
    fn Test_Is_Answered_Should_Be_True_For_Read_And_Approximate_Only()
    {
        assert!(Outcome::Read.Is_Answered());
        assert!(Outcome::Approximate.Is_Answered());
        assert!(!Outcome::Unreachable.Is_Answered());
    }
}
