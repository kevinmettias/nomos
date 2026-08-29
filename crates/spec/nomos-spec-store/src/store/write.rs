//! Writing content into the schema through a caller's transaction.

mod blocks;
mod documents;
mod relations;

pub(crate) use blocks::Write_Source_Blocks;
pub(crate) use documents::{Write_Blob, Write_Node, Write_Source_Document};
pub(crate) use relations::{Inverse_Of, Write_Relation};

/// Every row a mapped query produced, or the first failure it hit.
pub(crate) fn Collected_Rows<Row>(
    rows: impl Iterator<Item = rusqlite::Result<Row>>,
) -> Result<Vec<Row>, crate::StoreError>
{
    let mut collected = Vec::new();
    for row in rows
    {
        collected.push(row?);
    }

    return Ok(collected);
}
