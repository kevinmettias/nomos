//! [`PayloadRefusal`], bytes that are not this schema.

/// Why a payload's bytes could not be read as this schema.
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

impl std::error::Error for PayloadRefusal
{}
