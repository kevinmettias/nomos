//! Why an item is being declined.

/// Why an item is being declined, distinguished from [`crate::Holder`] for the reason given
/// there.
#[derive(Clone, Copy, Debug)]
pub struct DeclineReason<'a>(&'a str);

impl<'a, Text> From<&'a Text> for DeclineReason<'a>
where
    Text: AsRef<str> + ?Sized,
{
    fn from(value: &'a Text) -> Self
    {
        return DeclineReason(value.as_ref());
    }
}

impl<'a> DeclineReason<'a>
{
    /// The reason as a plain string.
    #[must_use]
    pub fn As_Text(&self) -> &'a str
    {
        return self.0;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_As_Text_Should_Return_The_Wrapped_String_Unchanged()
    {
        let reason = "superseded".to_owned();
        let decline_reason: DeclineReason<'_> = DeclineReason::from(&reason);

        assert_eq!(decline_reason.As_Text(), "superseded");
    }
}
