//! The stored corpus, at every grain a source document was segmented into.

use super::super::{Columns, Connection, Filter, Item, Name, ProjectError, Query, Value};

pub(crate) fn Gather_Suites(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT suite_id, title, authority_root FROM suites WHERE 1 = 1",
    );
    query.Prefix("suite_id", filter.identifier_prefix.as_ref());

    return query.Ordered_By("suite_id").Run(connection, |row| {
        let mut columns = Columns::Of(row);
        let suite = columns.Text()?;
        let title = columns.Text()?;
        let root: i64 = columns.Next()?;

        return Ok(Item::Of(&suite)
            .With(Name("title"), Value(&title))
            .With(Name("authority"), Value(if root == 1 { "root" } else { "sibling" })));
    });
}

pub(crate) fn Gather_Documents(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT d.path, d.revision, b.sha256,
                (SELECT count(*) FROM source_blocks WHERE document_uid = d.uid),
                (SELECT count(*) FROM source_headings WHERE document_uid = d.uid)
         FROM source_documents d JOIN blobs b ON b.uid = d.blob_uid
         WHERE 1 = 1",
    );
    query.Equal("d.path", filter.document.as_ref());
    query.Equal("d.revision", filter.revision.as_ref());

    return query.Ordered_By("d.revision, d.path").Run(connection, |row| {
        let mut columns = Columns::Of(row);
        let path = columns.Text()?;
        let revision = columns.Text()?;
        let hash = columns.Text()?;
        let blocks: i64 = columns.Next()?;
        let headings: i64 = columns.Next()?;

        return Ok(Item::Of(&format!("{path}@{revision}"))
            .With(Name("path"), Value(&path))
            .With(Name("revision"), Value(&revision))
            .With(Name("blocks"), Value(&blocks.to_string()))
            .With(Name("headings"), Value(&headings.to_string()))
            .With(Name("hash"), Value(&hash)));
    });
}

pub(crate) fn Gather_Headings(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT d.path, d.revision, h.ordinal, h.depth, h.title
         FROM source_headings h JOIN source_documents d ON d.uid = h.document_uid
         WHERE 1 = 1",
    );
    query.Equal("d.path", filter.document.as_ref());
    query.Equal("d.revision", filter.revision.as_ref());

    return query
        .Ordered_By("d.revision, d.path, h.ordinal")
        .Run(connection, |row| {
            let mut columns = Columns::Of(row);
            let path = columns.Text()?;
            let revision = columns.Text()?;
            let ordinal: i64 = columns.Next()?;
            let depth: i64 = columns.Next()?;
            let title = columns.Text()?;

            return Ok(Item::Of(&format!("{path}#{ordinal}"))
                .With(Name("revision"), Value(&revision))
                .With(Name("depth"), Value(&depth.to_string()))
                .With(Name("title"), Value(&title)));
        });
}

pub(crate) fn Gather_Blocks(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT d.path, d.revision, b.ordinal, b.kind, b.heading_path, b.text, b.content_hash
         FROM source_blocks b JOIN source_documents d ON d.uid = b.document_uid
         WHERE 1 = 1",
    );
    query.Equal("d.path", filter.document.as_ref());
    query.Equal("d.revision", filter.revision.as_ref());
    query.Equal("b.kind", filter.kind.as_ref());

    return query
        .Ordered_By("d.revision, d.path, b.ordinal")
        .Run(connection, |row| {
            let mut columns = Columns::Of(row);
            let path = columns.Text()?;
            let revision = columns.Text()?;
            let ordinal: i64 = columns.Next()?;
            let kind = columns.Text()?;
            let heading = columns.Text()?;
            let text = columns.Text()?;
            let hash = columns.Text()?;

            return Ok(Item::Of(&format!("{path}#{ordinal}"))
                .With(Name("revision"), Value(&revision))
                .With(Name("kind"), Value(&kind))
                .With(Name("heading"), Value(&heading))
                .With(Name("hash"), Value(&hash))
                .Carrying(&text));
        });
}

pub(crate) fn Gather_Rows(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT d.path, d.revision, b.ordinal, r.ordinal, r.table_ordinal, r.kind,
                r.cells_json, r.text, r.content_hash
         FROM source_table_rows r
         JOIN source_blocks b ON b.uid = r.source_block_uid
         JOIN source_documents d ON d.uid = b.document_uid
         WHERE 1 = 1",
    );
    query.Equal("d.path", filter.document.as_ref());
    query.Equal("d.revision", filter.revision.as_ref());
    query.Equal("r.kind", filter.row_kind.as_ref());

    return query
        .Ordered_By("d.revision, d.path, b.ordinal, r.ordinal")
        .Run(connection, |row| {
            let mut columns = Columns::Of(row);
            let path = columns.Text()?;
            let revision = columns.Text()?;
            let block: i64 = columns.Next()?;
            let ordinal: i64 = columns.Next()?;
            let table: i64 = columns.Next()?;
            let kind = columns.Text()?;
            let cells_json = columns.Text()?;
            let cells: Vec<String> = serde_json::from_str(&cells_json).unwrap_or_default();
            let text = columns.Text()?;
            let hash = columns.Text()?;

            return Ok(Item::Of(&format!("{path}#{block}:{ordinal}"))
                .With(Name("revision"), Value(&revision))
                .With(Name("kind"), Value(&kind))
                .With(Name("table"), Value(&table.to_string()))
                .With(Name("cells"), Value(&cells.join(" | ")))
                .With(Name("hash"), Value(&hash))
                .Carrying(&text));
        });
}
