//! The tier a relation type is declared at.

/// The tier a relation type is declared at (`seed`, `core`, `extended`, ...).
#[derive(Clone, Copy, Debug)]
pub struct RelationTier<'a>(pub &'a str);

impl<'a> From<&'a str> for RelationTier<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}

impl<'a> From<&'a String> for RelationTier<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Self(value.as_str());
    }
}
