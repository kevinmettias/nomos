//! [`TableLineResponse`], carried only by [`super::table::SpecTableResponse`].

use nomos_spec_store::TableLine;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_store::TableLine`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
pub struct TableLineResponse
{
    /// Which block of the document carries it.
    pub block_ordinal: u32,
    /// Which table within that block.
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

impl TableLineResponse
{
    pub(crate) fn From(line: TableLine) -> Self
    {
        return Self {
            block_ordinal: line.block_ordinal,
            table_ordinal: line.table_ordinal,
            row_ordinal: line.row_ordinal,
            kind: line.kind,
            cells: line.cells,
            text: line.text,
            content_hash: line.content_hash,
        };
    }
}
