//! Where a block's content went: one row per disposition, at the grain it was recorded at.

use super::super::{Columns, Connection, Filter, Item, Name, ProjectError, Query, Value};
use super::citation::Cited_Source;

/// Every lineage row with all three of its source kinds joined.
///
/// One query rather than a union of three, because a row carries exactly one source and a
/// `coalesce` over the three is what lets a single ordering cover all of them. `-1` is what
/// each unmatched join leaves behind, and is read back as "this row is not addressed at
/// that grain".
const LINEAGE_ROWS: &str = "SELECT l.disposition,
            coalesce(d.path, hd.path, rd.path, ''),
            coalesce(b.ordinal, rb.ordinal, -1), coalesce(r.ordinal, -1), coalesce(h.title, ''),
            coalesce(n.node_id, ''), coalesce(st.statement_id, '')
     FROM lineage l
     LEFT JOIN source_blocks b ON b.uid = l.source_block_uid
     LEFT JOIN source_documents d ON d.uid = b.document_uid
     LEFT JOIN source_headings h ON h.uid = l.source_heading_uid
     LEFT JOIN source_documents hd ON hd.uid = h.document_uid
     LEFT JOIN source_table_rows r ON r.uid = l.source_table_row_uid
     LEFT JOIN source_blocks rb ON rb.uid = r.source_block_uid
     LEFT JOIN source_documents rd ON rd.uid = rb.document_uid
     LEFT JOIN nodes n ON n.uid = l.target_node_uid
     LEFT JOIN normative_statements st ON st.uid = l.target_statement
     WHERE 1 = 1";

/// Document, then position, then disposition, then target.
///
/// Every column is named rather than ordering by the document alone, because two rows on
/// one block would otherwise come back in whatever order the join produced them and a
/// projection has to render the same way twice.
const LINEAGE_ORDER: &str = concat!(
    "coalesce(d.path, hd.path, rd.path, ''), coalesce(b.ordinal, -1), ",
    "coalesce(r.ordinal, -1), l.disposition, coalesce(n.node_id, ''), ",
    "coalesce(st.statement_id, '')",
);

pub(crate) fn Gather_Lineage(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(LINEAGE_ROWS);
    query.Equal("l.disposition", filter.disposition.as_ref());
    query.Equal("coalesce(d.path, hd.path, rd.path, '')", filter.document.as_ref());

    return query
        .Ordered_By(LINEAGE_ORDER)
        .Run(connection, |row| {
            let mut columns = Columns::Of(row);
            let disposition = columns.Text()?;
            let source = Cited_Source(&mut columns)?;
            // `node` is read before `statement` because the query names them in that order
            // and `Columns` reads positionally; both are read unconditionally regardless of
            // which one `target` below turns out to need.
            let node = columns.Text()?;
            let statement = columns.Text()?;
            // A statement is the more specific of the two and wins where both are present:
            // saying which node a block preserved is true but answers a coarser question
            // than the one the lineage was recorded to answer.
            let target = match statement.is_empty()
            {
                true => node,
                false => statement,
            };

            return Ok(Item::Of(&format!("{source} -> {disposition}"))
                .With(Name("source"), Value(&source))
                .With(Name("disposition"), Value(&disposition))
                .With(Name("target"), Value(&target)));
        });
}
