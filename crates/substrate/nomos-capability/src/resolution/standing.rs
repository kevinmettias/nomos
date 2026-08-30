//! How a chosen offer stands against what was asked for.

use nomos_contracts::Guarantee;
/// How one guarantee stands against another.
///
/// Four states rather than an ordering, because [`Guarantee::Satisfies`] is a preorder:
/// two guarantees may each reach everything the other does, and two may each reach
/// something the other does not. Collapsing those two into one "not stronger" is how a
/// registry comes to report a decision it did not make.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing
{
    /// Reaches everything the other reaches, and something it does not.
    Stronger,
    /// The other reaches everything this reaches, and something this does not.
    Weaker,
    /// Each reaches everything the other does. Two offers making the same promise, which
    /// the guarantee cannot tell apart and neither can this.
    Equivalent,
    /// Neither reaches everything the other does. Each is better on some axis, and which
    /// one that makes preferable is not a question a guarantee can answer.
    Incomparable,
}

impl Standing
{
    /// How `offered` stands against `against`.
    #[must_use]
    pub fn Of(offered: &Guarantee, against: &Guarantee) -> Self
    {
        return match (offered.Satisfies(against), against.Satisfies(offered))
        {
            (true, true) => Self::Equivalent,
            (true, false) => Self::Stronger,
            (false, true) => Self::Weaker,
            (false, false) => Self::Incomparable,
        };
    }

    /// Whether the guarantee separated the two at all.
    ///
    /// The question a composition root asks when it wants its provider choice to be a
    /// decision rather than a coincidence.
    #[must_use]
    pub const fn Is_Decided(self) -> bool
    {
        return matches!(self, Self::Stronger | Self::Weaker);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Assurance, FactVariant, IncrementalGranularity};

    fn Parse() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    fn Scan() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Approximate,
            Assurance::Unsound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    #[test]
    fn Test_Of_Should_Separate_Stronger_Weaker_And_Equivalent()
    {
        assert_eq!(Standing::Of(&Parse(), &Scan()), Standing::Stronger);
        assert_eq!(Standing::Of(&Scan(), &Parse()), Standing::Weaker);
        assert_eq!(Standing::Of(&Parse(), &Parse()), Standing::Equivalent);
    }

    #[test]
    fn Test_Is_Decided_Should_Be_True_Only_For_Stronger_Or_Weaker()
    {
        assert!(Standing::Stronger.Is_Decided());
        assert!(Standing::Weaker.Is_Decided());
        assert!(!Standing::Equivalent.Is_Decided());
        assert!(!Standing::Incomparable.Is_Decided());
    }
}
