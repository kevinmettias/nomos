//! What the corpus dropped and why: one row per omission, cited at the grain it carries.

use super::super::{Columns, Connection, Filter, Item, Name, ProjectError, Query, Row, Value};

/// Every omission with both of its source kinds joined.
///
/// `-1` is what the unmatched join leaves behind, and is read back as "this omission is not
/// addressed at that grain".
const OMISSION_ROWS: &str = "SELECT coalesce(d.path, hd.path, ''), coalesce(b.ordinal, -1),
            coalesce(h.title, ''), o.reason, o.justification, o.decision_record
     FROM omissions o
     LEFT JOIN source_blocks b ON b.uid = o.source_block_uid
     LEFT JOIN source_documents d ON d.uid = b.document_uid
     LEFT JOIN source_headings h ON h.uid = o.source_heading_uid
     LEFT JOIN source_documents hd ON hd.uid = h.document_uid
     WHERE 1 = 1";

/// Decision record, then document, then position, then reason — every column named, so two
/// omissions on one block cannot come back in whatever order the join produced them.
const OMISSION_ORDER: &str =
    "o.decision_record, coalesce(d.path, hd.path, ''), coalesce(b.ordinal, -1), o.reason";

pub(crate) fn Gather_Omissions(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(OMISSION_ROWS);
    query.Equal("coalesce(d.path, hd.path, '')", filter.document.as_ref());

    return query.Ordered_By(OMISSION_ORDER).Run(connection, An_Omission);
}

/// One omission, cited at the finest grain its row carries.
fn An_Omission(row: &Row<'_>) -> rusqlite::Result<Item>
{
    let mut columns = Columns::Of(row);
    let path = columns.Text()?;
    let block: i64 = columns.Next()?;
    let heading = columns.Text()?;
    let reason = columns.Text()?;
    let justification = columns.Text()?;
    let decision = columns.Text()?;
    let source = if block == -1
    {
        format!("{path}#{heading}")
    }
    else
    {
        format!("{path}#{block}")
    };

    return Ok(Item::Of(&format!("{source} -> {decision}"))
        .With(Name("source"), Value(&source))
        .With(Name("reason"), Value(&reason))
        .With(Name("justification"), Value(&justification))
        .With(Name("decision"), Value(&decision)));
}
