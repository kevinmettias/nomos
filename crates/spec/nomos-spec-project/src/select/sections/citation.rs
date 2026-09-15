//! Where a row points, at the finest grain it carries.

use super::super::Columns;

/// Where a lineage row points, at the finest grain the row carries.
///
/// A row addresses a table row, a block, or a heading, and `-1` is the sentinel each join
/// leaves behind when it matched nothing. Citing the block for a row-level disposition
/// would make thirty rows of one table cite the same place.
pub(crate) fn Cited_Source(columns: &mut Columns<'_, '_>) -> rusqlite::Result<String>
{
    let path = columns.Text()?;
    let block: i64 = columns.Next()?;
    let ordinal: i64 = columns.Next()?;
    let heading = columns.Text()?;

    return Ok(match (block, ordinal)
    {
        (-1, -1) => format!("{path}#{heading}"),
        (block, -1) => format!("{path}#{block}"),
        (block, ordinal) => format!("{path}#{block}:{ordinal}"),
    });
}
