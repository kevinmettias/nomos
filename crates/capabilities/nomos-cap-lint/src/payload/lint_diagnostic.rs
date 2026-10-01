//! One diagnostic, as the tool itself reported it.

use super::lint_level::LintLevel;

/// One diagnostic, as the tool itself reported it.
///
/// No span beyond a line: `OD-HOST-003` already declined to invent a column-precise
/// `Span` type anywhere in this workspace, and a lint tool's own primary span is exactly
/// the "where an editor needs one" case that record leaves to the editor's own language
/// server, not to a fact this capability materializes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintDiagnostic
{
    pub level: LintLevel,
    /// The tool's own name for this diagnostic's rule, if it named one — `clippy::
    /// needless_return`, for instance. `None` for a diagnostic the tool reported with no
    /// such identity; not every compiler diagnostic carries one.
    pub lint: Option<String>,
    /// The tool's own summary, single-line: an embedded newline is collapsed to a space
    /// by whichever provider encodes this, the same normalization
    /// `nomos-agent-executor-claude-code::Single_Line` applies for the identical reason — one line,
    /// one record.
    pub message: String,
    /// Repository-relative, forward slashes — the same convention every other subject in
    /// this workspace is addressed by.
    pub file: String,
    /// The diagnostic's primary span, one-based, as the tool itself reported it.
    pub line: u32,
}
