//! One reader per kind of section a profile can ask for.
//!
//! The readers are grouped by what they read: the stored corpus at every grain it was
//! segmented into, the record graph, what a node declares, lineage, and omissions. Nothing
//! below this module knows which file a reader lives in -- `select` and `query` name them
//! through this facade, and the tests below it call them the same way.

mod citation;
mod corpus;
mod graph;
mod lineage;
mod normative;
mod omissions;

pub(super) use corpus::{Gather_Blocks, Gather_Documents, Gather_Headings, Gather_Rows, Gather_Suites};
pub(super) use graph::{Gather_Families, Gather_Neighbourhood, Gather_Nodes};
pub(super) use lineage::Gather_Lineage;
pub(super) use normative::{Gather_Relations, Gather_Statements};
pub(super) use omissions::Gather_Omissions;

#[cfg(test)]
mod tests
{
    use super::citation::Cited_Source;
    use super::super::Columns;
    use super::*;
    use crate::Filter;
    use crate::Item;
    use nomos_spec_store::SpecificationStore;

    #[test]
    fn Test_Gather_Suites_Should_Read_Every_Suite_As_An_Item()
    {
        let store = Store_With(
            "INSERT INTO suites (suite_id, title, authority_root) \
             VALUES ('nomos', 'The Nomos specification', 1);",
        );

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
        let store = Store_With(
            "INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at)
             VALUES ('CDM-ONE', 'concept', 'canonical', 'record', 'One', NULL);
             INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at)
             VALUES ('CDM-GONE', 'concept', 'canonical', 'record', 'Gone', '2026-01-01T00:00:00Z');",
        );

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
        let store = Store_With(
            "INSERT INTO nodes (node_id, kind, authority, representation, title)
             VALUES ('AGT-EXEC-001', 'requirement', 'canonical', 'record', 'Ancestry');
             INSERT INTO normative_statements
                 (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
             SELECT uid, 'AGT-EXEC-001', 'requirement', 'Nomos shall record ancestry.',
                    'sha256:aa', NULL FROM nodes WHERE node_id = 'AGT-EXEC-001';",
        );

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
        let store = Store_With(
            "INSERT INTO nodes (node_id, kind, authority, representation, title)
             VALUES ('CDM-ONE', 'concept', 'canonical', 'record', 'One'),
                    ('AGT-EXEC-001', 'requirement', 'canonical', 'record', 'Ancestry');
             INSERT INTO relation_types
                 (name, tier, inverse_of, domain_kinds_json, range_kinds_json, max_per_node)
             VALUES ('verifies', 'core', NULL, '[\"concept\"]', '[\"requirement\"]', 8);
             INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
             SELECT f.uid, 'verifies', t.uid FROM nodes f, nodes t
             WHERE f.node_id = 'CDM-ONE' AND t.node_id = 'AGT-EXEC-001';",
        );

        let items = Gather_Relations(store.Connection(), &Filter::default()).expect("gathers");
        let relation = The_One_Item(&items);

        assert_eq!(relation.Field("from"), Some("CDM-ONE"));
        assert_eq!(relation.Field("to"), Some("AGT-EXEC-001"));
    }

    #[test]
    fn Test_Gather_Lineage_Should_Read_What_A_Block_Became()
    {
        let store = Store_With_A_Block(
            "INSERT INTO nodes (node_id, kind, authority, representation, title)
             VALUES ('CDM-ONE', 'concept', 'canonical', 'record', 'One');
             INSERT INTO lineage (source_block_uid, disposition, target_node_uid)
             SELECT b.uid, 'preserved-verbatim', n.uid FROM source_blocks b, nodes n
             WHERE n.node_id = 'CDM-ONE';",
        );

        let items = Gather_Lineage(store.Connection(), &Filter::default()).expect("gathers");
        let lineage = The_One_Item(&items);

        assert_eq!(lineage.Field("disposition"), Some("preserved-verbatim"));
        assert_eq!(lineage.Field("target"), Some("CDM-ONE"));
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
        let store = Store_With_A_Block(
            "INSERT INTO omissions (source_block_uid, reason, justification, decision_record)
             SELECT uid, 'superseded', 'replaced by the v15 records', 'D-129' FROM source_blocks;",
        );

        let items = Gather_Omissions(store.Connection(), &Filter::default()).expect("gathers");
        let omission = The_One_Item(&items);

        assert_eq!(omission.Field("reason"), Some("superseded"));
        assert_eq!(omission.Field("decision"), Some("D-129"));
    }

    /// An in-memory store seeded by `sql` alone — the setup every `Gather_*` test that needs no
    /// document shares.
    fn Store_With(sql: &str) -> SpecificationStore
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store.Connection().execute_batch(sql).expect("seeds");
        return store;
    }

    /// The one item a `Gather_*` returned, asserting on the way that it is the only one.
    ///
    /// The count and the unwrap that depends on it are one step. A test that checks the length
    /// and then takes the first item is stating the same fact twice, and the message `expect`
    /// raises belongs on the assertion that established the fact rather than on the read that
    /// followed it.
    fn The_One_Item(items: &[Item]) -> &Item
    {
        assert_eq!(items.len(), 1);

        return items.first().expect("asserted above to contain exactly one item");
    }

    /// An in-memory store holding one document and its one segmented block, seeded further by
    /// `sql` — the setup every `Gather_*` test that needs a block already loaded shares.
    fn Store_With_A_Block(sql: &str) -> SpecificationStore
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let document = store.Put_Source_Document("volumes/one.md", "v1", "Body text.\n").expect("stores");
        store.Put_Source_Blocks(document, &nomos_spec_model::Segment("Body text.\n")).expect("stores");
        store.Connection().execute_batch(sql).expect("seeds");
        return store;
    }
}
