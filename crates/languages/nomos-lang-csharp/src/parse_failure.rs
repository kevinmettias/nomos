//! A file the parser could not read, and where it stopped.

/// Why a file could not be read.
///
/// Carries where, because the point of this variant is that somebody can act on it. A corpus
/// walk that reports "some files unparseable" and cannot say which or where has produced a
/// number nobody can do anything with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseFailure
{
    pub line: usize,
    pub column: usize,
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
