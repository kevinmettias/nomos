//! Why a payload's bytes could not be decoded.

/// A `nomos.requirement.trace.v1` payload's bytes did not decode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    pub reason: String,
}

impl core::fmt::Display for Refusal
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}
