//! Whether a suite is this repository's own specification or one it merely references.

/// Whether a suite is this repository's own specification or one it merely references.
///
/// Named rather than a bool. `Put_Suite(suite_id, title, true)` said nothing at the call
/// site about what was true, and the distinction it carries is the one the store exists to
/// keep: a sibling's statement is quoted, not governed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuiteAuthority
{
    /// This repository's own specification, whose records govern here.
    Root,
    /// A suite read in from elsewhere, present so its nodes can be pointed at.
    Sibling,
}

impl SuiteAuthority
{
    /// How the `authority_root` column spells this.
    pub(crate) const fn Stored(self) -> i64
    {
        return match self
        {
            Self::Root => 1,
            Self::Sibling => 0,
        };
    }

    /// What the `authority_root` column meant.
    pub(crate) const fn Read(stored: i64) -> Self
    {
        return if stored == 0 { Self::Sibling } else { Self::Root };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Stored_Should_Spell_Root_As_One_And_Sibling_As_Zero()
    {
        assert_eq!(SuiteAuthority::Root.Stored(), 1);
        assert_eq!(SuiteAuthority::Sibling.Stored(), 0);
    }

    #[test]
    fn Test_Read_Should_Recover_What_The_Column_Meant()
    {
        assert_eq!(SuiteAuthority::Read(1), SuiteAuthority::Root);
        assert_eq!(SuiteAuthority::Read(0), SuiteAuthority::Sibling);
    }
}
