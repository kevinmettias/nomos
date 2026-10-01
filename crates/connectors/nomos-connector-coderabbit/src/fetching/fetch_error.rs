//! Why `gh api` could not be read as one review comment's own GitHub response.

/// The vendor could not be reached, or answered with something other than a fetched review
/// comment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchError
{
    pub reason: String,
}

impl core::fmt::Display for FetchError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}", self.reason);
    }
}
