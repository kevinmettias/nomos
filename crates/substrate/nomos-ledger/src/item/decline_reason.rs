//! Why an item is being declined.

/// Why an item is being declined, distinguished from [`crate::Holder`] for the reason given
/// there.
#[derive(Clone, Copy, Debug)]
pub struct DeclineReason<'a>(&'a str);

impl<'a> From<&'a str> for DeclineReason<'a>
{
    fn from(value: &'a str) -> Self
    {
        return DeclineReason(value);
    }
}

impl<'a> From<&'a String> for DeclineReason<'a>
{
    fn from(value: &'a String) -> Self
    {
        return DeclineReason(value.as_str());
    }
}

impl<'a> DeclineReason<'a>
{
    /// The reason as a plain string.
    #[must_use]
    pub fn As_Str(&self) -> &'a str
    {
        return self.0;
    }
}
