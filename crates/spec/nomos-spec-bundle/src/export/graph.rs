//! Reading the graph out: the nodes, what joins them, and what they were restored from.
//!
//! The collectors themselves live in [`collect`], one per record kind. They stay beside
//! the tests that exercise them rather than in a sibling file, because a test's companion
//! attribution follows the file it is textually written in.

mod collect;

pub(super) use collect::{
    Collect_Lineages, Collect_Nodes, Collect_Omissions, Collect_Relations, Collect_Repeated_Text_Declarations,
    Collect_Suites, Node_Aliases, Node_Histories, Normative_Statements, Record_Front_Matter, Record_Relations,
    Relation_Types,
};

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::DocumentRef;
    use crate::FrontMatter as RecordFrontMatter;
    use crate::OrdinalRef;
    use crate::Record;
    use nomos_spec_store::SpecificationStore;

    #[test]
    fn Test_Collect_Suites_Should_Read_A_Suite_By_Its_Natural_Key()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Collect_Suites(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::Suite(crate::Suite {
                suite_id: "nomos".to_owned(),
                title: "The Nomos Specification".to_owned(),
                authority_root: true,
            })]
        );
    }

    #[test]
    fn Test_Collect_Nodes_Should_Read_A_Node_And_The_Suite_It_Belongs_To()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Collect_Nodes(store.Connection(), &mut records).expect("Collect_Nodes ran over the store Fixture filled");

        assert_eq!(records, The_Nodes_It_Holds());
    }

    /// The two nodes the fixture seeds, with the suite that only the first of them names.
    fn The_Nodes_It_Holds() -> Vec<Record>
    {
        return vec![
            Record::Node(crate::Node {
                node_id: "N1".to_owned(),
                kind: "requirement".to_owned(),
                authority: "canonical".to_owned(),
                representation: "record".to_owned(),
                title: "Node One".to_owned(),
                deleted_at: None,
                suite_id: Some("nomos".to_owned()),
            }),
            Record::Node(crate::Node {
                node_id: "N2".to_owned(),
                kind: "concept".to_owned(),
                authority: "canonical".to_owned(),
                representation: "record".to_owned(),
                title: "Node Two".to_owned(),
                deleted_at: None,
                suite_id: None,
            }),
        ];
    }

    #[test]
    fn Test_Node_Aliases_Should_Read_An_Alias_By_The_Node_It_Names()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Node_Aliases(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::NodeAlias(crate::Alias {
                alias: "N1-OLD".to_owned(),
                node_id: "N1".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Node_Histories_Should_Read_An_Event_By_The_Node_It_Belongs_To()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Node_Histories(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::NodeHistory(crate::History {
                node_id: "N1".to_owned(),
                ordinal: 1,
                event: "created".to_owned(),
                reason: "seeded".to_owned(),
                previous_event_hash: None,
                event_hash: "sha256:01".to_owned(),
                recorded_at: "2026-01-01T00:00:00Z".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Relation_Types_Should_Decode_The_Domain_And_Range_Json_Columns()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Relation_Types(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::RelationType(crate::Type {
                name: "verifies".to_owned(),
                tier: "core".to_owned(),
                inverse_of: None,
                domain: Vec::new(),
                range: Vec::new(),
                max_per_node: FIXTURE_MAX_PER_NODE,
            })]
        );
    }

    #[test]
    fn Test_Collect_Relations_Should_Read_Both_Ends_By_Their_Node_Id()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Collect_Relations(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::Relation(crate::Relation {
                from_node_id: "N2".to_owned(),
                relation_type: "verifies".to_owned(),
                to_node_id: "N1".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Normative_Statements_Should_Read_A_Statement_By_The_Node_It_Belongs_To()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Normative_Statements(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::NormativeStatement(crate::NormativeStatement {
                statement_id: "STMT-1".to_owned(),
                node_id: "N1".to_owned(),
                kind: "Requirement".to_owned(),
                canonical_text: "Text".to_owned(),
                canonical_hash: "sha256:aa".to_owned(),
                supersedes_hash: None,
            })]
        );
    }

    #[test]
    fn Test_Collect_Lineages_Should_Read_A_Heading_Anchored_Row()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Collect_Lineages(store.Connection(), &mut records).expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::Lineage(crate::Lineage {
                source_block: None,
                source_heading: Some(OrdinalRef {
                    document: DocumentRef {
                        path: "doc.md".to_owned(),
                        revision: "v1".to_owned(),
                    },
                    ordinal: 1,
                }),
                source_table_row: None,
                disposition: "preserved-normalized".to_owned(),
                target_node_id: Some("N1".to_owned()),
                target_statement_id: None,
            })]
        );
    }

    #[test]
    fn Test_Collect_Omissions_Should_Read_A_Heading_Anchored_Row()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Collect_Omissions(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::Omission(crate::Omission {
                source_block: None,
                source_heading: Some(OrdinalRef {
                    document: DocumentRef {
                        path: "doc.md".to_owned(),
                        revision: "v1".to_owned(),
                    },
                    ordinal: 1,
                }),
                reason: "superseded".to_owned(),
                justification: "replaced".to_owned(),
                decision_record: "D-1".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Record_Front_Matter_Should_Decode_The_Tags_Json_Column()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Record_Front_Matter(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::RecordFrontMatter(RecordFrontMatter {
                document: DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                node_id: "N1".to_owned(),
                status: "accepted".to_owned(),
                version: 1,
                tags: Vec::new(),
            })]
        );
    }

    #[test]
    fn Test_Record_Relations_Should_Read_A_Declared_Relation_By_Its_Document()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Record_Relations(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::RecordRelation(crate::RecordRelation {
                document: DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                ordinal: 1,
                target: "N2".to_owned(),
                relation: "affects".to_owned(),
            })]
        );
    }

    /// The relation type `verifies`, as the fixture seeds it: no inverse, no domain or range
    /// kinds, and a `max_per_node` this test can tell apart from the column default.
    const FIXTURE_MAX_PER_NODE: u32 = 4;

    /// One row of everything this file reads: a suite, two nodes (one inside it and one
    /// not), an alias, a history entry, a relation type, a relation between the two nodes,
    /// a normative statement, a heading-anchored lineage row, a heading-anchored omission,
    /// and one record's declared front matter and one declared relation.
    const SEED_SQL: &str =
        "INSERT INTO blobs (sha256, byte_length, content) VALUES ('sha256:aa', 2, x'6869');
         INSERT INTO source_documents (path, revision, blob_uid) VALUES ('doc.md', 'v1', 1);
         INSERT INTO source_headings (document_uid, ordinal, depth, title)
             VALUES (1, 1, 1, 'Intro');
         INSERT INTO suites (suite_id, title, authority_root)
             VALUES ('nomos', 'The Nomos Specification', 1);
         INSERT INTO nodes
             (node_id, kind, authority, representation, title, deleted_at, suite_uid)
             VALUES ('N1', 'requirement', 'canonical', 'record', 'Node One', NULL, 1);
         INSERT INTO nodes
             (node_id, kind, authority, representation, title, deleted_at, suite_uid)
             VALUES ('N2', 'concept', 'canonical', 'record', 'Node Two', NULL, NULL);
         INSERT INTO node_aliases (alias, node_uid) VALUES ('N1-OLD', 1);
         INSERT INTO node_history
             (node_uid, ordinal, event, reason, previous_event_hash, event_hash, recorded_at)
             VALUES (1, 1, 'created', 'seeded', NULL, 'sha256:01', '2026-01-01T00:00:00Z');
         INSERT INTO relation_types
             (name, tier, inverse_of, domain_kinds_json, range_kinds_json, max_per_node)
             VALUES ('verifies', 'core', NULL, '[]', '[]', 4);
         INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
             VALUES (2, 'verifies', 1);
         INSERT INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
             VALUES (1, 'STMT-1', 'Requirement', 'Text', 'sha256:aa', NULL);
         INSERT INTO lineage (source_heading_uid, disposition, target_node_uid)
             VALUES (1, 'preserved-normalized', 1);
         INSERT INTO omissions (source_heading_uid, reason, justification, decision_record)
             VALUES (1, 'superseded', 'replaced', 'D-1');
         INSERT INTO record_front_matter (document_uid, node_uid, status, version, tags_json)
             VALUES (1, 1, 'accepted', 1, '[]');
         INSERT INTO record_relations (document_uid, ordinal, target, relation)
             VALUES (1, 1, 'N2', 'affects');";

    /// The store [`SEED_SQL`] fills.
    fn Fixture() -> SpecificationStore
    {
        let store = SpecificationStore::In_Memory()
            .expect("an in-memory store applies the schema this build carries");
        store
            .Connection()
            .execute_batch(SEED_SQL)
            .expect("populates every table this file reads");

        return store;
    }
}
