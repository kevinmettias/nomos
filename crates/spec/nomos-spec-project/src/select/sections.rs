//! One reader per kind of section a profile can ask for.

use super::{Connection, Filter, Item, ProjectError, Query, Text, Narrow_To_Nodes, Row};

pub(super) fn Suites(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT suite_id, title, authority_root FROM suites WHERE 1 = 1",
    );
    query.Prefix("suite_id", filter.identifier_prefix.as_ref());

    return query.Ordered_By("suite_id").Run(connection, |row| {
        let root: i64 = row.get(2)?;
        let suite = Text(row, 0)?;
        let title = Text(row, 1)?;

        return Ok(Item::Of(&suite)
            .With("title", &title)
            .With("authority", if root == 1 { "root" } else { "sibling" }));
    });
}

pub(super) fn Documents(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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
        let blocks: i64 = row.get(3)?;
        let headings: i64 = row.get(4)?;
        let path = Text(row, 0)?;
        let revision = Text(row, 1)?;
        let hash = Text(row, 2)?;

        return Ok(Item::Of(&format!("{path}@{revision}"))
            .With("path", &path)
            .With("revision", &revision)
            .With("blocks", &blocks.to_string())
            .With("headings", &headings.to_string())
            .With("hash", &hash));
    });
}

pub(super) fn Headings(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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
            let ordinal: i64 = row.get(2)?;
            let depth: i64 = row.get(3)?;
            let path = Text(row, 0)?;
            let revision = Text(row, 1)?;
            let title = Text(row, 4)?;

            return Ok(Item::Of(&format!("{path}#{ordinal}"))
                .With("revision", &revision)
                .With("depth", &depth.to_string())
                .With("title", &title));
        });
}

pub(super) fn Blocks(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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
            let ordinal: i64 = row.get(2)?;
            let path = Text(row, 0)?;
            let revision = Text(row, 1)?;
            let kind = Text(row, 3)?;
            let heading = Text(row, 4)?;
            let text = Text(row, 5)?;
            let hash = Text(row, 6)?;

            return Ok(Item::Of(&format!("{path}#{ordinal}"))
                .With("revision", &revision)
                .With("kind", &kind)
                .With("heading", &heading)
                .With("hash", &hash)
                .Carrying(&text));
        });
}

pub(super) fn Rows(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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
            let block: i64 = row.get(2)?;
            let ordinal: i64 = row.get(3)?;
            let table: i64 = row.get(4)?;
            let cells_json = Text(row, 6)?;
            let cells: Vec<String> = serde_json::from_str(&cells_json).unwrap_or_default();
            let path = Text(row, 0)?;
            let revision = Text(row, 1)?;
            let kind = Text(row, 5)?;
            let text = Text(row, 7)?;
            let hash = Text(row, 8)?;

            return Ok(Item::Of(&format!("{path}#{block}:{ordinal}"))
                .With("revision", &revision)
                .With("kind", &kind)
                .With("table", &table.to_string())
                .With("cells", &cells.join(" | "))
                .With("hash", &hash)
                .Carrying(&text));
        });
}

pub(super) fn Nodes(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT n.node_id, n.kind, n.authority, n.representation, n.title, s.suite_id
         FROM nodes n LEFT JOIN suites s ON s.uid = n.suite_uid
         WHERE n.deleted_at IS NULL",
    );
    Narrow_To_Nodes(&mut query, filter);

    return query.Ordered_By("n.node_id").Run(connection, |row| {
        let node = Text(row, 0)?;
        let kind = Text(row, 1)?;
        let authority = Text(row, 2)?;
        let representation = Text(row, 3)?;
        let title = Text(row, 4)?;
        let suite = Text(row, 5)?;

        return Ok(Item::Of(&node)
            .With("kind", &kind)
            .With("authority", &authority)
            .With("representation", &representation)
            .With("title", &title)
            .With("suite", &suite));
    });
}

pub(super) fn Statements(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT s.statement_id, s.kind, n.node_id, s.canonical_text, s.canonical_hash,
                s.supersedes_hash
         FROM normative_statements s JOIN nodes n ON n.uid = s.node_uid
         WHERE 1 = 1",
    );
    query.Equal("s.kind", filter.kind.as_ref());
    query.Prefix("s.statement_id", filter.identifier_prefix.as_ref());
    query.Equal("n.node_id", filter.node_id.as_ref());

    return query.Ordered_By("s.statement_id").Run(connection, |row| {
        let statement = Text(row, 0)?;
        let kind = Text(row, 1)?;
        let node = Text(row, 2)?;
        let text = Text(row, 3)?;
        let hash = Text(row, 4)?;
        let supersedes = Text(row, 5)?;

        return Ok(Item::Of(&statement)
            .With("kind", &kind)
            .With("node", &node)
            .With("supersedes", &supersedes)
            .With("hash", &hash)
            .Carrying(&text));
    });
}

pub(super) fn Relations(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT f.node_id, r.relation_type, t.node_id, y.tier, s.suite_id
         FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         JOIN relation_types y ON y.name = r.relation_type
         LEFT JOIN suites s ON s.uid = f.suite_uid
         WHERE 1 = 1",
    );
    query.Equal("r.relation_type", filter.relation_type.as_ref());
    query.Equal("s.suite_id", filter.suite.as_ref());
    query.Prefix("f.node_id", filter.identifier_prefix.as_ref());
    query.Either("f.node_id", "t.node_id", filter.node_id.as_ref());

    return query
        .Ordered_By("f.node_id, r.relation_type, t.node_id")
        .Run(connection, |row| {
            let from = Text(row, 0)?;
            let relation = Text(row, 1)?;
            let to = Text(row, 2)?;
            let tier = Text(row, 3)?;

            return Ok(Item::Of(&format!("{from} {relation} {to}"))
                .With("from", &from)
                .With("relation", &relation)
                .With("to", &to)
                .With("tier", &tier));
        });
}

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
const LINEAGE_ORDER: &str = "coalesce(d.path, hd.path, rd.path, ''), coalesce(b.ordinal, -1), \
     coalesce(r.ordinal, -1), l.disposition, coalesce(n.node_id, ''), \
     coalesce(st.statement_id, '')";

pub(super) fn Lineage(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(LINEAGE_ROWS);
    query.Equal("l.disposition", filter.disposition.as_ref());
    query.Equal("coalesce(d.path, hd.path, rd.path, '')", filter.document.as_ref());

    return query
        .Ordered_By(LINEAGE_ORDER)
        .Run(connection, |row| {
            let disposition = Text(row, 0)?;
            let source = Cited(row)?;
            let node = Text(row, 5)?;
            let statement = Text(row, 6)?;
            // A statement is the more specific of the two and wins where both are present:
            // saying which node a block preserved is true but answers a coarser question
            // than the one the lineage was recorded to answer.
            let target = if statement.is_empty() { node } else { statement };

            return Ok(Item::Of(&format!("{source} -> {disposition}"))
                .With("source", &source)
                .With("disposition", &disposition)
                .With("target", &target));
        });
}

/// Where a lineage row points, at the finest grain the row carries.
///
/// A row addresses a table row, a block, or a heading, and `-1` is the sentinel each join
/// leaves behind when it matched nothing. Citing the block for a row-level disposition
/// would make thirty rows of one table cite the same place.
pub(super) fn Cited(row: &Row<'_>) -> rusqlite::Result<String>
{
    let block: i64 = row.get(2)?;
    let ordinal: i64 = row.get(3)?;
    let path = Text(row, 1)?;
    let heading = Text(row, 4)?;

    return Ok(match (block, ordinal)
    {
        (-1, -1) => format!("{path}#{heading}"),
        (block, -1) => format!("{path}#{block}"),
        (block, ordinal) => format!("{path}#{block}:{ordinal}"),
    });
}

pub(super) fn Omissions(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT coalesce(d.path, hd.path, ''), coalesce(b.ordinal, -1),
                coalesce(h.title, ''), o.reason, o.justification, o.decision_record
         FROM omissions o
         LEFT JOIN source_blocks b ON b.uid = o.source_block_uid
         LEFT JOIN source_documents d ON d.uid = b.document_uid
         LEFT JOIN source_headings h ON h.uid = o.source_heading_uid
         LEFT JOIN source_documents hd ON hd.uid = h.document_uid
         WHERE 1 = 1",
    );
    query.Equal("coalesce(d.path, hd.path, '')", filter.document.as_ref());

    return query
        .Ordered_By(
            "o.decision_record, coalesce(d.path, hd.path, ''), coalesce(b.ordinal, -1), o.reason",
        )
        .Run(connection, |row| {
            let block: i64 = row.get(1)?;
            let path = Text(row, 0)?;
            let heading = Text(row, 2)?;
            let reason = Text(row, 3)?;
            let justification = Text(row, 4)?;
            let decision = Text(row, 5)?;
            let source = if block == -1
            {
                format!("{path}#{heading}")
            }
            else
            {
                format!("{path}#{block}")
            };

            return Ok(Item::Of(&format!("{source} -> {decision}"))
                .With("source", &source)
                .With("reason", &reason)
                .With("justification", &justification)
                .With("decision", &decision));
        });
}
