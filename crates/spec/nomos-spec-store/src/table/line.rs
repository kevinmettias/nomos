//! One line of a table, as stored.

/// One line of a table, as stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line
{
    /// Which block of the document carries it.
    pub block_ordinal: u32,
    /// Which table within that block. A block may carry more than one.
    pub table_ordinal: u32,
    /// Which line within the block, 1-based.
    pub row_ordinal: u32,
    /// `header`, `content` or `separator`.
    pub kind: String,
    pub cells: Vec<String>,
    /// The line as authored.
    pub text: String,
    pub content_hash: String,
}
