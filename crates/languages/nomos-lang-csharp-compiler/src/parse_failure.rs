//! [`ParseFailure`], a file whose directives the compiler would refuse.

/// Why a file's directives cannot be judged, and on which line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseFailure
{
    /// The one-based line the refusal is about.
    pub line: usize,
    /// What is wrong there.
    pub message: String,
}

impl core::fmt::Display for ParseFailure
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "line {}: {}", self.line, self.message);
    }
}

impl std::error::Error for ParseFailure
{}
