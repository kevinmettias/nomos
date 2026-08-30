//! [`TableLineResponse`], carried only by [`super::table_response::TableResponse`].

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Copy_Every_Field_Of_The_Domain_Table_Line()
    {
        let line = TableLine {
            block_ordinal: 2,
            table_ordinal: 1,
            row_ordinal: 3,
            kind: "content".to_owned(),
            cells: vec!["a".to_owned(), "b".to_owned()],
            text: "| a | b |".to_owned(),
            content_hash: "abc123".to_owned(),
        };

        let response = TableLineResponse::From(line.clone());

        assert_eq!(response.block_ordinal, line.block_ordinal);
        assert_eq!(response.table_ordinal, line.table_ordinal);
        assert_eq!(response.row_ordinal, line.row_ordinal);
        assert_eq!(response.kind, line.kind);
        assert_eq!(response.cells, line.cells);
        assert_eq!(response.text, line.text);
        assert_eq!(response.content_hash, line.content_hash);
    }
}
