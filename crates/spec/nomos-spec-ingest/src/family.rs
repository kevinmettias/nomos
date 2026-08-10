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

    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[Self::Requirement, Self::Story, Self::Acceptance];
    }
}
