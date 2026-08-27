//! The identifier of a relation edge's target node.

/// The identifier of a relation edge's target node.
#[derive(Clone, Copy, Debug)]
pub struct ToNodeId<'a>(pub &'a str);

impl<'a> From<&'a str> for ToNodeId<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}

impl<'a> From<&'a String> for ToNodeId<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Self(value.as_str());
    }
}
