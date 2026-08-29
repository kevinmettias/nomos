//! The title a suite is filed under.

/// The title a suite is filed under.
#[derive(Clone, Copy, Debug)]
pub struct SuiteTitle<'a>(pub &'a str);

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
