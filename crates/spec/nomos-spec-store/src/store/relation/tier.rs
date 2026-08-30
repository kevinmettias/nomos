//! The tier a relation type is declared at.

/// The tier a relation type is declared at (`seed`, `core`, `extended`, ...).
#[derive(Clone, Copy, Debug)]
pub struct Tier<'a>(pub &'a str);

impl<'a> From<&'a str> for Tier<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}

impl<'a> From<&'a String> for Tier<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Self(value.as_str());
    }
}
