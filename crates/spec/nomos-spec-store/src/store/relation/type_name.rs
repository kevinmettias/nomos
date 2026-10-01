//! The name a relation type is registered, declared, or paired under.

/// The name of a relation type: what an edge is registered under, what a type is declared
/// as, or which type a pairing names as the one gaining an inverse.
#[derive(Clone, Copy, Debug)]
pub struct TypeName<'a>(pub &'a str);

impl<'a> From<&'a str> for TypeName<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}
