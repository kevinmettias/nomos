//! Inserting the graph: the nodes, what joins them, and what they were restored from.
//!
//! The inserts themselves live in [`insert`], one per record kind. They stay beside the
//! tests that exercise them rather than in a sibling file, because a test's companion
//! attribution follows the file it is textually written in.

mod insert;

pub(super) use insert::{
    Insert_Lineage, Insert_Node_Aliases, Insert_Node_History, Insert_Nodes,
    Insert_Normative_Statements, Insert_Omissions, Insert_Record_Front_Matter,
    Insert_Record_Relations, Insert_Relation_Types, Insert_Relations, Insert_Suites,
};

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Bundle;
    use crate::BundleError;
    use crate::Record;
    use nomos_spec_store::SpecificationStore;
    use rusqlite::Transaction;

    #[test]
    fn Test_Insert_Suites_Should_Place_A_Suite_Row()
    {
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::Suite(crate::Suite {
                suite_id: "nomos".to_owned(),
                title: "The Nomos Specification".to_owned(),
                authority_root: true,
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Suites(transaction, &bundle)).expect("inserts");

        let row: (String, String, i64) = store
            .Connection()
            .query_row("SELECT suite_id, title, authority_root FROM suites", [], |row| {
                return Ok((row.get(0)?, row.get(1)?, row.get(2)?));
            })
            .expect("reads back");
        assert_eq!(row, ("nomos".to_owned(), "The Nomos Specification".to_owned(), 1));
    }

    #[test]
    fn Test_Insert_Nodes_Should_Place_A_Node_With_No_Suite()
    {
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::Node(crate::Node {
                node_id: "N3".to_owned(),
                kind: "requirement".to_owned(),
                authority: "canonical".to_owned(),
                representation: "record".to_owned(),
                title: "Node Three".to_owned(),
                deleted_at: None,
                suite_id: None,
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Nodes(transaction, &bundle)).expect("inserts");

        let kind: String = store
            .Connection()
            .query_row("SELECT kind FROM nodes WHERE node_id = 'N3'", [], |row| row.get(0))
            .expect("reads back");
        assert_eq!(kind, "requirement");
    }

    #[test]
    fn Test_Insert_Node_Aliases_Should_Place_An_Alias_By_Node_Id()
    {
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::NodeAlias(crate::Alias {
                alias: "N1-OLD".to_owned(),
                node_id: "N1".to_owned(),
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Node_Aliases(transaction, &bundle)).expect("inserts");

        let node_id: String = store
            .Connection()
            .query_row(
                "SELECT n.node_id FROM node_aliases a JOIN nodes n ON n.uid = a.node_uid
                 WHERE a.alias = 'N1-OLD'",
                [],
                |row| row.get(0),
            )
            .expect("reads back");
        assert_eq!(node_id, "N1");
    }

    #[test]
    fn Test_Insert_Node_History_Should_Place_An_Event_By_Node_Id()
    {
        let event = Placed_Value(
            Record::NodeHistory(crate::History {
                node_id: "N1".to_owned(),
                ordinal: 1,
                event: "created".to_owned(),
                reason: "seeded".to_owned(),
                previous_event_hash: None,
                event_hash: "sha256:01".to_owned(),
                recorded_at: "2026-01-01T00:00:00Z".to_owned(),
            }),
            Insert_Node_History,
            "SELECT h.event FROM node_history h JOIN nodes n ON n.uid = h.node_uid
             WHERE n.node_id = 'N1'",
        );
        assert_eq!(event, "created");
    }

    #[test]
    fn Test_Insert_Relation_Types_Should_Place_A_Relation_Type_Row()
    {
        let tier = Placed_Value(
            Record::RelationType(crate::Type {
                name: "depends_on".to_owned(),
                tier: "core".to_owned(),
                inverse_of: None,
                domain: Vec::new(),
                range: Vec::new(),
                max_per_node: 4,
            }),
            Insert_Relation_Types,
            "SELECT tier FROM relation_types WHERE name = 'depends_on'",
        );
        assert_eq!(tier, "core");
    }

    #[test]
    fn Test_Insert_Relations_Should_Join_Both_Ends_By_Node_Id()
    {
        let to_node_id = Placed_Value(
            Record::Relation(crate::Relation {
                from_node_id: "N1".to_owned(),
                relation_type: "verifies".to_owned(),
                to_node_id: "N2".to_owned(),
            }),
            Insert_Relations,
            "SELECT t.node_id FROM relations r
             JOIN nodes f ON f.uid = r.from_node_uid
             JOIN nodes t ON t.uid = r.to_node_uid
             WHERE f.node_id = 'N1' AND r.relation_type = 'verifies'",
        );
        assert_eq!(to_node_id, "N2");
    }

    #[test]
    fn Test_Insert_Normative_Statements_Should_Place_A_Statement_By_Node_Id()
    {
        let canonical_text = Placed_Value(
            Record::NormativeStatement(crate::NormativeStatement {
                statement_id: "STMT-1".to_owned(),
                node_id: "N1".to_owned(),
                kind: "Requirement".to_owned(),
                canonical_text: "Text".to_owned(),
                canonical_hash: "sha256:aa".to_owned(),
                supersedes_hash: None,
            }),
            Insert_Normative_Statements,
            "SELECT canonical_text FROM normative_statements WHERE statement_id = 'STMT-1'",
        );
        assert_eq!(canonical_text, "Text");
    }

    #[test]
    fn Test_Insert_Lineage_Should_Place_A_Heading_Anchored_Row()
    {
        let disposition = Placed_Value(
            Record::Lineage(crate::Lineage {
                source_block: None,
                source_heading: Some(crate::OrdinalRef {
                    document: crate::DocumentRef {
                        path: "doc.md".to_owned(),
                        revision: "v1".to_owned(),
                    },
                    ordinal: 1,
                }),
                source_table_row: None,
                disposition: "preserved-normalized".to_owned(),
                target_node_id: None,
                target_statement_id: None,
            }),
            Insert_Lineage,
            "SELECT disposition FROM lineage WHERE source_heading_uid = 1",
        );
        assert_eq!(disposition, "preserved-normalized");
    }

    #[test]
    fn Test_Insert_Omissions_Should_Place_A_Heading_Anchored_Row()
    {
        let reason = Placed_Value(
            Record::Omission(crate::Omission {
                source_block: None,
                source_heading: Some(crate::OrdinalRef {
                    document: crate::DocumentRef {
                        path: "doc.md".to_owned(),
                        revision: "v1".to_owned(),
                    },
                    ordinal: 1,
                }),
                reason: "superseded".to_owned(),
                justification: "replaced".to_owned(),
                decision_record: "D-1".to_owned(),
            }),
            Insert_Omissions,
            "SELECT reason FROM omissions WHERE source_heading_uid = 1",
        );
        assert_eq!(reason, "superseded");
    }

    #[test]
    fn Test_Insert_Record_Front_Matter_Should_Decode_The_Tags_Json_Column()
    {
        let tags_json = Placed_Value(
            Record::RecordFrontMatter(crate::FrontMatter {
                document: crate::DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                node_id: "N1".to_owned(),
                status: "accepted".to_owned(),
                version: 1,
                tags: vec!["architecture".to_owned()],
            }),
            Insert_Record_Front_Matter,
            "SELECT tags_json FROM record_front_matter WHERE document_uid = 1",
        );
        assert_eq!(tags_json, "[\"architecture\"]");
    }

    #[test]
    fn Test_Insert_Record_Relations_Should_Place_A_Declared_Relation_By_Document()
    {
        let target = Placed_Value(
            Record::RecordRelation(crate::RecordRelation {
                document: crate::DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                ordinal: 1,
                target: "N2".to_owned(),
                relation: "affects".to_owned(),
            }),
            Insert_Record_Relations,
            "SELECT target FROM record_relations WHERE document_uid = 1",
        );
        assert_eq!(target, "N2");
    }

    /// Places a bundle holding exactly one record with `insert`, then reads back the single
    /// column `sql` names — the value the caller's own assertion is about.
    ///
    /// The record is the caller's, so a test states only which row it places and which
    /// column proves it; the placing and the reading back are the same for every kind.
    fn Placed_Value(
        record: Record,
        insert: fn(&Transaction<'_>, &Bundle) -> Result<(), BundleError>,
        sql: &str,
    ) -> String
    {
        let mut store = Fixture();
        let bundle =
            Bundle::New(1, vec![record]).expect("the bundle carries exactly the record the caller placed");

        store
            .In_Transaction(|transaction| insert(transaction, &bundle))
            .expect("the insert places the row the caller reads back");

        return store
            .Connection()
            .query_row(sql, [], |row| row.get(0))
            .expect("the column the caller named is readable from the row it placed");
    }

    /// A blob, the document read from it and one heading inside it, and two nodes and one
    /// relation type this file's functions can resolve their references against.
    fn Fixture() -> SpecificationStore
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO blobs (sha256, byte_length, content) VALUES ('sha256:aa', 2, x'6869');
                 INSERT INTO source_documents (path, revision, blob_uid) VALUES ('doc.md', 'v1', 1);
                 INSERT INTO source_headings (document_uid, ordinal, depth, title)
                     VALUES (1, 1, 1, 'Intro');
                 INSERT INTO nodes
                     (node_id, kind, authority, representation, title, deleted_at, suite_uid)
                     VALUES ('N1', 'requirement', 'canonical', 'record', 'Node One', NULL, NULL);
                 INSERT INTO nodes
                     (node_id, kind, authority, representation, title, deleted_at, suite_uid)
                     VALUES ('N2', 'concept', 'canonical', 'record', 'Node Two', NULL, NULL);
                 INSERT INTO relation_types
                     (name, tier, inverse_of, domain_kinds_json, range_kinds_json, max_per_node)
                     VALUES ('verifies', 'core', NULL, '[]', '[]', 4);",
            )
            .expect("populates every table this file's inserts resolve against");

        return store;
    }
}
