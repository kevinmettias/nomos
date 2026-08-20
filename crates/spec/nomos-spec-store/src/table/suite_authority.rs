//! Whether a suite is this repository's own specification or one it merely references.

/// A suite's own identifier.
///
/// Distinct from [`SuiteTitle`] even though both are `&str`, because the two sit adjacent
/// at `Put_Suite`'s call site and a position is not a name — a transposed pair used to
/// still compile.
#[derive(Clone, Copy, Debug)]
pub struct SuiteId<'a>(pub &'a str);

/// The title a suite is filed under.
#[derive(Clone, Copy, Debug)]
pub struct SuiteTitle<'a>(pub &'a str);

impl<'a> From<&'a str> for SuiteId<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}

impl<'a> From<&'a String> for SuiteId<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Self(value.as_str());
    }
}

impl<'a> From<&'a str> for SuiteTitle<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}

impl<'a> From<&'a String> for SuiteTitle<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Self(value.as_str());
    }
}

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
