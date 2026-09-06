//! Combining a file and a tool-reported line into an editor-grade span, at projection time.
//!
//! `nomos_cap_lint::LintDiagnostic`'s own doc states the design point this crate must not
//! cross: "no span beyond a line... a lint tool's own primary span is exactly the 'where an
//! editor needs one' case that record [`OD-HOST-003`] leaves to the editor's own language
//! server, not to a fact this capability materializes." This function is that combination.
//! It never touches `nomos-cap-lint` or widens what a fact carries; it takes the one line a
//! [`crate::location::Location`] already carried and turns it into an [`lsp_types::Range`]
//! spanning the whole line, because nothing this crate reads names a column.

use lsp_types::{Position, Range};

/// The whole-line range for `line` (one-based, as every rule in this workspace reports it),
/// or the first line of the file when nothing more specific was available.
///
/// LSP positions are zero-based; `line.saturating_sub(1)` is the one conversion. A finding
/// carrying no line at all is not a caller error -- `crate::location::Location::Parse`
/// documents which rules never carry one -- so `None` renders as line zero rather than
/// refusing to produce a range at all: a diagnostic anchored at the top of the file the
/// reader already opened is more useful than none.
#[must_use]
pub(crate) fn Range_For(line: Option<u32>) -> Range
{
    let zero_based = line.unwrap_or(1).saturating_sub(1);
    let next_line = zero_based.saturating_add(1);

    return Range { start: Position { line: zero_based, character: 0 }, end: Position { line: next_line, character: 0 } };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Range_For_Should_Convert_A_One_Based_Line_To_A_Zero_Based_Whole_Line_Span()
    {
        let range = Range_For(Some(42));

        assert_eq!(range.start, Position { line: 41, character: 0 });
        assert_eq!(range.end, Position { line: 42, character: 0 });
    }

    #[test]
    fn Test_Range_For_Should_Anchor_At_The_Top_Of_The_File_When_No_Line_Was_Reported()
    {
        let range = Range_For(None);

        assert_eq!(range.start, Position { line: 0, character: 0 });
        assert_eq!(range.end, Position { line: 1, character: 0 });
    }
}
