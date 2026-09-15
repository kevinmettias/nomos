//! One place in the workspace a verdict is about.
//!
//! Ported from `tests/contract/tests/requirement_trace/assessment.rs`. Its own file, because
//! every scope here declares one public type and this is the one an entry names many of.

/// A place in the workspace a verdict is about.
///
/// A path alone would nearly never fire: files are renamed far less often than the symbols
/// inside them. The symbol is what makes the entry decay visibly when the thing it was
/// about is renamed out from under it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Site
{
    /// Repo-relative, forward slashes.
    pub path: String,
    /// Text that must occur in that file.
    pub symbol: String,
}
