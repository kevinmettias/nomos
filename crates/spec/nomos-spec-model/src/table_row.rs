//! One line of a markdown table, held as a child of the block that carries it.

use crate::normalize::ContentHash;
use crate::row_kind::RowKind;

/// One line of a markdown table, held as a child of the block that carries it.
///
/// The block remains the preservation authority: it stores the verbatim text and both
/// v14 hashes, so byte completeness never depends on rows. Rows exist so a loss report
/// can name what went missing by identity rather than by count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableRow
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

impl TableRow
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
