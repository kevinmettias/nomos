//! How widely a declaration is visible, as written.

/// The visibility an item declares.
///
/// Three values, not `nomos-lang-rust`'s four. Go has no restricted middle ground — no
/// `pub(crate)`, no `pub(in path)` — visibility is a two-way fact about the identifier
/// itself: capitalized is exported, everything else is not, and that rule applies uniformly
/// to every named declaration Go has, with no separate exemption for an interface method the
/// way Rust exempts a trait method. [`Visibility::NotApplicable`] survives here for a
/// narrower, genuinely Go-specific reason: the blank identifier `_`, legal wherever Go
/// permits a declared name, and used most often as a compile-time interface assertion
/// (`var _ Writer = (*File)(nil)`). `_` binds nothing, so it is not a name in the sense
/// visibility is a fact about — recording it as `Private` would claim the file restricted
/// access to something the file gave no access to in the first place.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Visibility
{
    /// The identifier's first letter is uppercase.
    Public,
    /// The identifier's first letter is lowercase, or an underscore followed by a letter.
    Private,
    /// The blank identifier `_` — a form that declares no name to have visibility.
    NotApplicable,
}

impl Visibility
{
    /// The stable label used in an encoded payload.
    #[must_use]
    pub fn Label(&self) -> &'static str
    {
        return match self
        {
            Self::Public => nomos_cap_syntax::PUBLIC,
            Self::Private => "Private",
            Self::NotApplicable => nomos_cap_syntax::NOT_APPLICABLE,
        };
    }

    /// Decides visibility from an identifier exactly as Go's own spec does: by its first
    /// Unicode letter's case. No filesystem access and no name resolution — the same
    /// syntactic-only promise every other judgment in this crate makes.
    #[must_use]
    pub fn Of_Name(name: &str) -> Self
    {
        if name == "_"
        {
            return Self::NotApplicable;
        }

        return match name.chars().next()
        {
            Some(first) if first.is_uppercase() => Self::Public,
            _ => Self::Private,
        };
    }
}

impl core::fmt::Display for Visibility
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Of_Name_Should_Treat_A_Capitalized_Name_As_Public()
    {
        assert_eq!(Visibility::Of_Name("Add"), Visibility::Public);
    }

    #[test]
    fn Test_A_Lowercase_Name_Should_Be_Private()
    {
        assert_eq!(Visibility::Of_Name("add"), Visibility::Private);
    }

    #[test]
    fn Test_An_Underscore_Prefixed_Name_Should_Be_Private()
    {
        assert_eq!(Visibility::Of_Name("_scratch"), Visibility::Private);
    }

    #[test]
    fn Test_The_Blank_Identifier_Should_Declare_No_Visibility()
    {
        assert_eq!(Visibility::Of_Name("_"), Visibility::NotApplicable);
    }

    #[test]
    fn Test_Label_Should_Match_The_Shared_Vocabulary()
    {
        assert_eq!(Visibility::Public.Label(), nomos_cap_syntax::PUBLIC);
        assert_eq!(Visibility::NotApplicable.Label(), nomos_cap_syntax::NOT_APPLICABLE);
    }
}
