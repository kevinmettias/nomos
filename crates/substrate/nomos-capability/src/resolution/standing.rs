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
