//! A payload could not be decoded under its schema.

/// A payload could not be decoded under this schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayloadRefusal
{
    pub reason: String,
}

impl core::fmt::Display for PayloadRefusal
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}
