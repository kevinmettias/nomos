/// Where a document lives, as a repository-relative path.
#[derive(Clone, Copy, Debug)]
pub struct DocumentPath<'a>(pub &'a str);

impl<'a> From<&'a str> for DocumentPath<'a>
{
    fn from(value: &'a str) -> Self
    {
        return Self(value);
    }
}

impl<'a> From<&'a String> for DocumentPath<'a>
{
    fn from(value: &'a String) -> Self
    {
        return Self(value.as_str());
    }
}
