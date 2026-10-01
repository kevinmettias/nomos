//! Why GitHub's own response could not be read as this connector's canonical fields.

/// GitHub's own response could not be read as this reader expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranslationError
{
    pub reason: String,
}

impl core::fmt::Display for TranslationError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}
