//! The relation type a pairing names as the one gaining an inverse.

/// The relation type a `TypeName` is paired with as its inverse.
#[derive(Clone, Copy, Debug)]
pub struct InverseRelationType<'a>(pub &'a str);

impl<'a> From<&'a str> for InverseRelationType<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}

impl<'a> From<&'a String> for InverseRelationType<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Self(value.as_str());
    }
}
