//! Which revision of a document was read.

/// Which revision of the corpus (or [`crate::AUTHORED`]) a document was read at.
#[derive(Clone, Copy, Debug)]
pub struct DocumentRevision<'a>(pub &'a str);

impl<'a> From<&'a str> for DocumentRevision<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}

impl<'a> From<&'a String> for DocumentRevision<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Self(value.as_str());
    }
}
