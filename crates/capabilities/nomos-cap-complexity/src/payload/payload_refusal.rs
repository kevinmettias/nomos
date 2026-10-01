//! [`PayloadRefusal`], why a payload could not be read.

/// Why bytes stamped with this capability's schema could not be read as a
/// [`super::complexity_payload::ComplexityPayload`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PayloadRefusal
{
    /// What was wrong, naming the line where there is one.
    pub reason: String,
}

impl core::fmt::Display for PayloadRefusal
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}
