//! One line of a markdown table, held as a child of the block that carries it.

use crate::ContentHash;
use crate::RowKind;

/// One line of a markdown table, held as a child of the block that carries it.
///
/// The block remains the preservation authority: it stores the verbatim text and both
/// v14 hashes, so byte completeness never depends on rows. Rows exist so a loss report
/// can name what went missing by identity rather than by count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row
{
    /// 1-based within the block.
    pub ordinal: u32,
    /// 1-based within the block. A block may carry more than one table.
    pub table_ordinal: u32,
    pub kind: RowKind,
    pub cells: Vec<String>,
    /// The line as authored.
    pub text: String,
}

impl Row
{
    #[must_use]
    pub fn Content_Hash(&self) -> ContentHash
    {
        return ContentHash::Of(&self.text);
    }

    #[must_use]
    pub fn Normalized_Hash(&self) -> ContentHash
    {
        return ContentHash::Of_Normalized(&self.text);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn Row_With_Text(text: &str) -> Row
    {
        return Row {
            ordinal: 1,
            table_ordinal: 1,
            kind: RowKind::Content,
            cells: Vec::new(),
            text: text.to_owned(),
        };
    }

    #[test]
    fn Test_Content_Hash_Should_Hash_The_Rows_Text_Verbatim()
    {
        let row = Row_With_Text("| a | b |");

        assert_eq!(row.Content_Hash(), ContentHash::Of("| a | b |"));
    }

    #[test]
    fn Test_Normalized_Hash_Should_Hash_The_Rows_Normalized_Text()
    {
        let row = Row_With_Text("| a  |  b |");

        assert_eq!(row.Normalized_Hash(), ContentHash::Of_Normalized("| a  |  b |"));
        assert_ne!(row.Normalized_Hash(), row.Content_Hash(), "the whitespace run must collapse");
    }
}
