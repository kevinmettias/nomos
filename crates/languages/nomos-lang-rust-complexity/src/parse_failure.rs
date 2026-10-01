//! [`ParseFailure`], a file the parser could not read and where it stopped.

/// Why a file could not be read, and where, so somebody can act on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseFailure
{
    /// The one-based line the parser stopped on.
    pub line: usize,
    /// The zero-based column the parser stopped on.
    pub column: usize,
    /// What the parser said.
    pub message: String,
}

impl core::fmt::Display for ParseFailure
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "line {}, column {}: {}", self.line, self.column, self.message);
    }
}

impl std::error::Error for ParseFailure
{}
