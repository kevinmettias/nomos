//! One reader per kind of section a profile can ask for.

use super::{
    Columns, Connection, Filter, FirstColumn, Item, Name, ProjectError, Query, Narrow_To_Nodes, Row,
    SecondColumn, Value,
};

pub(super) fn Gather_Suites(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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

pub(super) fn Gather_Documents(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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

pub(super) fn Gather_Headings(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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

pub(super) fn Gather_Blocks(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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

pub(super) fn Gather_Rows(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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

pub(super) fn Gather_Nodes(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT n.node_id, n.kind, n.authority, n.representation, n.title, s.suite_id
         FROM nodes n LEFT JOIN suites s ON s.uid = n.suite_uid
         WHERE n.deleted_at IS NULL",
    );
    Narrow_To_Nodes(&mut query, filter);

    return query.Ordered_By("n.node_id").Run(connection, |row| {
        let mut columns = Columns::Of(row);
        let node = columns.Text()?;
        let kind = columns.Text()?;
        let authority = columns.Text()?;
        let representation = columns.Text()?;
        let title = columns.Text()?;
        let suite = columns.Text()?;

        return Ok(Item::Of(&node)
            .With(Name("kind"), Value(&kind))
            .With(Name("authority"), Value(&authority))
            .With(Name("representation"), Value(&representation))
            .With(Name("title"), Value(&title))
            .With(Name("suite"), Value(&suite)));
    });
}

pub(super) fn Gather_Statements(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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
        let mut columns = Columns::Of(row);
        let statement = columns.Text()?;
        let kind = columns.Text()?;
        let node = columns.Text()?;
        let text = columns.Text()?;
        let hash = columns.Text()?;
        let supersedes = columns.Text()?;

        return Ok(Item::Of(&statement)
            .With(Name("kind"), Value(&kind))
            .With(Name("node"), Value(&node))
            .With(Name("supersedes"), Value(&supersedes))
            .With(Name("hash"), Value(&hash))
            .Carrying(&text));
    });
}

pub(super) fn Gather_Relations(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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
    query.Either(FirstColumn("f.node_id"), SecondColumn("t.node_id"), filter.node_id.as_ref());

    return query
        .Ordered_By("f.node_id, r.relation_type, t.node_id")
        .Run(connection, |row| {
            let mut columns = Columns::Of(row);
            let from = columns.Text()?;
            let relation = columns.Text()?;
            let to = columns.Text()?;
            let tier = columns.Text()?;

            return Ok(Item::Of(&format!("{from} {relation} {to}"))
                .With(Name("from"), Value(&from))
                .With(Name("relation"), Value(&relation))
                .With(Name("to"), Value(&to))
                .With(Name("tier"), Value(&tier)));
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

pub(super) fn Gather_Lineage(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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

/// Where a lineage row points, at the finest grain the row carries.
///
/// A row addresses a table row, a block, or a heading, and `-1` is the sentinel each join
/// leaves behind when it matched nothing. Citing the block for a row-level disposition
/// would make thirty rows of one table cite the same place.
pub(super) fn Cited_Source(columns: &mut Columns<'_, '_>) -> rusqlite::Result<String>
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

pub(super) fn Gather_Omissions(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_store::SpecificationStore;

    #[test]
    fn Test_Gather_Suites_Should_Read_Every_Suite_As_An_Item()
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO suites (suite_id, title, authority_root) \
                 VALUES ('nomos', 'The Nomos specification', 1);",
            )
            .expect("seeds");

        let items = Gather_Suites(store.Connection(), &Filter::default()).expect("gathers");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").identity,
            "nomos"
        );
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("authority"),
            Some("root")
        );
    }

    #[test]
    fn Test_Gather_Documents_Should_Read_A_Documents_Path_And_Revision()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        store.Put_Source_Document("volumes/one.md", "v1", "Body text.\n").expect("stores");

        let items = Gather_Documents(store.Connection(), &Filter::default()).expect("gathers");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("path"),
            Some("volumes/one.md")
        );
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("revision"),
            Some("v1")
        );
    }

    #[test]
    fn Test_Gather_Headings_Should_Read_A_Headings_Title()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let document = store.Put_Source_Document("volumes/one.md", "v1", "# One\n").expect("stores");
        store
            .Connection()
            .execute_batch(&format!(
                "INSERT INTO source_headings (document_uid, ordinal, depth, title) \
                 VALUES ({document}, 1, 1, 'One');"
            ))
            .expect("seeds");

        let items = Gather_Headings(store.Connection(), &Filter::default()).expect("gathers");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("title"),
            Some("One")
        );
    }

    #[test]
    fn Test_Gather_Blocks_Should_Read_A_Blocks_Kind_And_Text()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let document = store.Put_Source_Document("volumes/one.md", "v1", "Body text.\n").expect("stores");
        store
            .Put_Source_Blocks(document, &nomos_spec_model::Segment("Body text.\n"))
            .expect("stores");

        let items = Gather_Blocks(store.Connection(), &Filter::default()).expect("gathers");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("kind"),
            Some("prose")
        );
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").body.as_deref(),
            Some("Body text.")
        );
    }

    #[test]
    fn Test_Gather_Rows_Should_Read_A_Content_Rows_Cells()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let table = "| a | b |\n| --- | --- |\n| 1 | 2 |\n";
        let document = store.Put_Source_Document("volumes/one.md", "v1", table).expect("stores");
        store.Put_Source_Blocks(document, &nomos_spec_model::Segment(table)).expect("stores");

        let filter = Filter {
            row_kind: Some("content".to_owned()),
            ..Filter::default()
        };
        let items = Gather_Rows(store.Connection(), &filter).expect("gathers");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("cells"),
            Some("1 | 2")
        );
    }

    #[test]
    fn Test_Gather_Nodes_Should_Ignore_A_Deleted_Node()
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at)
                 VALUES ('CDM-ONE', 'concept', 'canonical', 'record', 'One', NULL);
                 INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at)
                 VALUES ('CDM-GONE', 'concept', 'canonical', 'record', 'Gone', '2026-01-01T00:00:00Z');",
            )
            .expect("seeds");

        let items = Gather_Nodes(store.Connection(), &Filter::default()).expect("gathers");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").identity,
            "CDM-ONE"
        );
    }

    #[test]
    fn Test_Gather_Statements_Should_Read_A_Statement_And_Its_Node()
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO nodes (node_id, kind, authority, representation, title)
                 VALUES ('AGT-EXEC-001', 'requirement', 'canonical', 'record', 'Ancestry');
                 INSERT INTO normative_statements
                     (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
                 SELECT uid, 'AGT-EXEC-001', 'requirement', 'Nomos shall record ancestry.',
                        'sha256:aa', NULL FROM nodes WHERE node_id = 'AGT-EXEC-001';",
            )
            .expect("seeds");

        let items = Gather_Statements(store.Connection(), &Filter::default()).expect("gathers");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("node"),
            Some("AGT-EXEC-001")
        );
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").body.as_deref(),
            Some("Nomos shall record ancestry.")
        );
    }

    #[test]
    fn Test_Gather_Relations_Should_Read_A_Relations_Two_Ends()
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO nodes (node_id, kind, authority, representation, title)
                 VALUES ('CDM-ONE', 'concept', 'canonical', 'record', 'One'),
                        ('AGT-EXEC-001', 'requirement', 'canonical', 'record', 'Ancestry');
                 INSERT INTO relation_types
                     (name, tier, inverse_of, domain_kinds_json, range_kinds_json, max_per_node)
                 VALUES ('verifies', 'core', NULL, '[\"concept\"]', '[\"requirement\"]', 8);
                 INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
                 SELECT f.uid, 'verifies', t.uid FROM nodes f, nodes t
                 WHERE f.node_id = 'CDM-ONE' AND t.node_id = 'AGT-EXEC-001';",
            )
            .expect("seeds");

        let items = Gather_Relations(store.Connection(), &Filter::default()).expect("gathers");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("from"),
            Some("CDM-ONE")
        );
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("to"),
            Some("AGT-EXEC-001")
        );
    }

    #[test]
    fn Test_Gather_Lineage_Should_Read_What_A_Block_Became()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let document = store.Put_Source_Document("volumes/one.md", "v1", "Body text.\n").expect("stores");
        store
            .Put_Source_Blocks(document, &nomos_spec_model::Segment("Body text.\n"))
            .expect("stores");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO nodes (node_id, kind, authority, representation, title)
                 VALUES ('CDM-ONE', 'concept', 'canonical', 'record', 'One');
                 INSERT INTO lineage (source_block_uid, disposition, target_node_uid)
                 SELECT b.uid, 'preserved-verbatim', n.uid FROM source_blocks b, nodes n
                 WHERE n.node_id = 'CDM-ONE';",
            )
            .expect("seeds");

        let items = Gather_Lineage(store.Connection(), &Filter::default()).expect("gathers");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("disposition"),
            Some("preserved-verbatim")
        );
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("target"),
            Some("CDM-ONE")
        );
    }

    #[test]
    fn Test_Cited_Source_Should_Address_A_Row_By_Block_And_Ordinal()
    {
        let connection = rusqlite::Connection::open_in_memory().expect("opens");
        connection
            .execute_batch(
                "CREATE TABLE t (path TEXT, block INTEGER, ordinal INTEGER, heading TEXT);
                 INSERT INTO t VALUES ('volumes/one.md', 3, 2, '');",
            )
            .expect("seeds");

        let cited = connection
            .query_row("SELECT path, block, ordinal, heading FROM t", [], |row| {
                let mut columns = Columns::Of(row);
                return Cited_Source(&mut columns);
            })
            .expect("reads");

        assert_eq!(cited, "volumes/one.md#3:2");
    }

    #[test]
    fn Test_Gather_Omissions_Should_Read_Why_A_Block_Was_Dropped()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let document = store.Put_Source_Document("volumes/one.md", "v1", "Body text.\n").expect("stores");
        store
            .Put_Source_Blocks(document, &nomos_spec_model::Segment("Body text.\n"))
            .expect("stores");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO omissions (source_block_uid, reason, justification, decision_record)
                 SELECT uid, 'superseded', 'replaced by the v15 records', 'D-129' FROM source_blocks;",
            )
            .expect("seeds");

        let items = Gather_Omissions(store.Connection(), &Filter::default()).expect("gathers");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("reason"),
            Some("superseded")
        );
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").Field("decision"),
            Some("D-129")
        );
    }
}
