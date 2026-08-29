//! The document families a v15 overlay reconciles.

/// The three v14 artifact families v15.0 was supposed to carry forward.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family
{
    Requirement,
    Story,
    Acceptance,
}

impl Family
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Requirement => "requirement",
            Self::Story => "story",
            Self::Acceptance => "acceptance",
        };
    }

    /// The v14 directory each family is authored in.
    #[must_use]
    pub const fn Directory(self) -> &'static str
    {
        return match self
        {
            Self::Requirement => "requirements",
            Self::Story => "stories",
            Self::Acceptance => "acceptance",
        };
    }

    /// Every family this pass reconciles.
    ///
    /// Mirrored by `Test_Every_Family_Should_Be_Matched_Exhaustively`, an exhaustive match
    /// over every variant with no wildcard arm, in this file. It fails to compile, not
    /// merely to pass, if a variant is added here without being added there.
    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[Self::Requirement, Self::Story, Self::Acceptance];
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// `Family::All()`'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to `Family` without a matching arm
    /// added here fails this file to *compile*, not merely to pass — the property D-134
    /// asks a closed enum's mirror to have.
    #[test]
    fn Test_Every_Family_Should_Be_Matched_Exhaustively()
    {
        fn Ordinal_Of(family: Family) -> usize
        {
            return match family
            {
                Family::Requirement => 0,
                Family::Story => 1,
                Family::Acceptance => 2,
            };
        }

        for (index, family) in Family::All().iter().enumerate()
        {
            assert_eq!(
                Ordinal_Of(*family),
                index,
                "{} is not matched at the position Family::All() puts it, so the exhaustive \
                 match and the universe have drifted apart",
                family.Label()
            );
        }
    }
}
