//! Inserting the graph: the nodes, what joins them, and what they were restored from.

// file-size: allow this file pairs its production code with its own inline #[cfg(test)]
// module; check-test-coverage keys a test's companion unit off the exact file it is
// textually written in, so these tests cannot move to a sibling file without losing
// their attribution to every function this file declares.
// responsibility: allow same reason -- the coupling that keeps this file whole is
// check-test-coverage's stem-based companion attribution, not a design choice.

use rusqlite::{Transaction, params};

use crate::BundleError;
use crate::Bundle;
use crate::Record;

use super::reference::{
    Document_Uid, Node_Uid, Optional_Block_Uid, Optional_Heading_Uid, Optional_Node_Uid,
    Optional_Statement_Uid, Optional_Suite_Uid, Optional_Table_Row_Uid,
};
use super::Insert_Each;

pub(super) fn Insert_Suites(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO suites (suite_id, title, authority_root) VALUES (?1, ?2, ?3)",
        |insert, record| {
            let Record::Suite(suite) = record
            else
            {
                return Ok(());
            };

            insert.execute(params![
                suite.suite_id,
                suite.title,
                i64::from(suite.authority_root)
            ])?;

            return Ok(());
        },
    );
}

pub(super) fn Insert_Nodes(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO nodes
         (node_id, kind, authority, representation, title, deleted_at, suite_uid)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        |insert, record| {
            let Record::Node(node) = record
            else
            {
                return Ok(());
            };

            let suite_uid = Optional_Suite_Uid(transaction, node.suite_id.as_deref())?;
            insert.execute(params![
                node.node_id,
                node.kind,
                node.authority,
                node.representation,
                node.title,
                node.deleted_at,
                suite_uid
            ])?;

            return Ok(());
        },
    );
}

pub(super) fn Insert_Node_Aliases(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    let mut insert =
        transaction.prepare("INSERT INTO node_aliases (alias, node_uid) VALUES (?1, ?2)")?;

    for record in bundle.Records()
    {
        let Record::NodeAlias(alias) = record
        else
        {
            continue;
        };

        let node_uid = Node_Uid(transaction, &alias.node_id)?;
        insert.execute(params![alias.alias, node_uid])?;
    }

    return Ok(());
}

pub(super) fn Insert_Node_History(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO node_history
         (node_uid, ordinal, event, reason, previous_event_hash, event_hash, recorded_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        |insert, record| {
            let Record::NodeHistory(entry) = record
            else
            {
                return Ok(());
            };

            let node_uid = Node_Uid(transaction, &entry.node_id)?;
            insert.execute(params![
                node_uid,
                entry.ordinal,
                entry.event,
                entry.reason,
                entry.previous_event_hash,
                entry.event_hash,
                entry.recorded_at
            ])?;

            return Ok(());
        },
    );
}

pub(super) fn Insert_Relation_Types(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    let mut insert = transaction.prepare(
        "INSERT INTO relation_types
             (name, tier, inverse_of, domain_kinds_json, range_kinds_json, max_per_node)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;

    for record in bundle.Records()
    {
        let Record::RelationType(relation_type) = record
        else
        {
            continue;
        };

        Insert_One_Relation_Type(&mut insert, relation_type)?;
    }

    return Ok(());
}

/// One relation type row, its domain and range encoded back to JSON.
fn Insert_One_Relation_Type(
    insert: &mut rusqlite::Statement<'_>,
    relation_type: &crate::Type,
) -> Result<(), BundleError>
{
    let domain_json = serde_json::to_string(&relation_type.domain)
        .map_err(|error| return BundleError::Sql(error.to_string()))?;
    let range_json = serde_json::to_string(&relation_type.range)
        .map_err(|error| return BundleError::Sql(error.to_string()))?;

    insert.execute(params![
        relation_type.name,
        relation_type.tier,
        relation_type.inverse_of,
        domain_json,
        range_json,
        relation_type.max_per_node,
    ])?;

    return Ok(());
}

pub(super) fn Insert_Relations(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
         VALUES (?1, ?2, ?3)",
        |insert, record| {
            let Record::Relation(relation) = record
            else
            {
                return Ok(());
            };

            let from_uid = Node_Uid(transaction, &relation.from_node_id)?;
            let to_uid = Node_Uid(transaction, &relation.to_node_id)?;
            insert.execute(params![from_uid, relation.relation_type, to_uid])?;

            return Ok(());
        },
    );
}

pub(super) fn Insert_Normative_Statements(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO normative_statements
         (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        |insert, record| {
            let Record::NormativeStatement(statement) = record
            else
            {
                return Ok(());
            };

            let node_uid = Node_Uid(transaction, &statement.node_id)?;
            insert.execute(params![
                node_uid,
                statement.statement_id,
                statement.kind,
                statement.canonical_text,
                statement.canonical_hash,
                statement.supersedes_hash
            ])?;

            return Ok(());
        },
    );
}

pub(super) fn Insert_Lineage(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO lineage
         (source_block_uid, source_heading_uid, source_table_row_uid, disposition,
          target_node_uid, target_statement)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        |insert, record| {
            let Record::Lineage(lineage) = record
            else
            {
                return Ok(());
            };
            let block_uid = Optional_Block_Uid(transaction, lineage.source_block.as_ref())?;
            let heading_uid = Optional_Heading_Uid(transaction, lineage.source_heading.as_ref())?;
            let row_uid = Optional_Table_Row_Uid(transaction, lineage.source_table_row.as_ref())?;
            let node_uid = Optional_Node_Uid(transaction, lineage.target_node_id.as_deref())?;
            let statement_uid =
                Optional_Statement_Uid(transaction, lineage.target_statement_id.as_deref())?;
            insert.execute(params![
                block_uid,
                heading_uid,
                row_uid,
                lineage.disposition,
                node_uid,
                statement_uid
            ])?;

            return Ok(());
        },
    );
}

pub(super) fn Insert_Omissions(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO omissions
         (source_block_uid, source_heading_uid, reason, justification, decision_record)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        |insert, record| {
            let Record::Omission(omission) = record
            else
            {
                return Ok(());
            };
            let block_uid = Optional_Block_Uid(transaction, omission.source_block.as_ref())?;
            let heading_uid = Optional_Heading_Uid(transaction, omission.source_heading.as_ref())?;
            insert.execute(params![
                block_uid,
                heading_uid,
                omission.reason,
                omission.justification,
                omission.decision_record
            ])?;

            return Ok(());
        },
    );
}

pub(super) fn Insert_Record_Front_Matter(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO record_front_matter
         (document_uid, node_uid, status, version, tags_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        |insert, record| {
            let Record::RecordFrontMatter(front_matter) = record
            else
            {
                return Ok(());
            };

            let tags = serde_json::to_string(&front_matter.tags)
                .map_err(|error| BundleError::Sql(error.to_string()))?;
            let document_uid = Document_Uid(transaction, &front_matter.document)?;
            let node_uid = Node_Uid(transaction, &front_matter.node_id)?;
            insert.execute(params![
                document_uid,
                node_uid,
                front_matter.status,
                front_matter.version,
                tags
            ])?;

            return Ok(());
        },
    );
}

pub(super) fn Insert_Record_Relations(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO record_relations (document_uid, ordinal, target, relation)
         VALUES (?1, ?2, ?3, ?4)",
        |insert, record| {
            let Record::RecordRelation(relation) = record
            else
            {
                return Ok(());
            };
            let document_uid = Document_Uid(transaction, &relation.document)?;
            insert.execute(params![
                document_uid,
                relation.ordinal,
                relation.target,
                relation.relation
            ])?;

            return Ok(());
        },
    );
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_store::SpecificationStore;

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
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::NodeHistory(crate::History {
                node_id: "N1".to_owned(),
                ordinal: 1,
                event: "created".to_owned(),
                reason: "seeded".to_owned(),
                previous_event_hash: None,
                event_hash: "sha256:01".to_owned(),
                recorded_at: "2026-01-01T00:00:00Z".to_owned(),
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Node_History(transaction, &bundle)).expect("inserts");

        let event: String = store
            .Connection()
            .query_row(
                "SELECT h.event FROM node_history h JOIN nodes n ON n.uid = h.node_uid
                 WHERE n.node_id = 'N1'",
                [],
                |row| row.get(0),
            )
            .expect("reads back");
        assert_eq!(event, "created");
    }

    #[test]
    fn Test_Insert_Relation_Types_Should_Place_A_Relation_Type_Row()
    {
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::RelationType(crate::Type {
                name: "depends_on".to_owned(),
                tier: "core".to_owned(),
                inverse_of: None,
                domain: Vec::new(),
                range: Vec::new(),
                max_per_node: 4,
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Relation_Types(transaction, &bundle)).expect("inserts");

        let tier: String = store
            .Connection()
            .query_row("SELECT tier FROM relation_types WHERE name = 'depends_on'", [], |row| {
                row.get(0)
            })
            .expect("reads back");
        assert_eq!(tier, "core");
    }

    #[test]
    fn Test_Insert_Relations_Should_Join_Both_Ends_By_Node_Id()
    {
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::Relation(crate::Relation {
                from_node_id: "N1".to_owned(),
                relation_type: "verifies".to_owned(),
                to_node_id: "N2".to_owned(),
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Relations(transaction, &bundle)).expect("inserts");

        let to_node_id: String = store
            .Connection()
            .query_row(
                "SELECT t.node_id FROM relations r
                 JOIN nodes f ON f.uid = r.from_node_uid
                 JOIN nodes t ON t.uid = r.to_node_uid
                 WHERE f.node_id = 'N1' AND r.relation_type = 'verifies'",
                [],
                |row| row.get(0),
            )
            .expect("reads back");
        assert_eq!(to_node_id, "N2");
    }

    #[test]
    fn Test_Insert_Normative_Statements_Should_Place_A_Statement_By_Node_Id()
    {
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::NormativeStatement(crate::NormativeStatement {
                statement_id: "STMT-1".to_owned(),
                node_id: "N1".to_owned(),
                kind: "Requirement".to_owned(),
                canonical_text: "Text".to_owned(),
                canonical_hash: "sha256:aa".to_owned(),
                supersedes_hash: None,
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Normative_Statements(transaction, &bundle)).expect("inserts");

        let canonical_text: String = store
            .Connection()
            .query_row(
                "SELECT canonical_text FROM normative_statements WHERE statement_id = 'STMT-1'",
                [],
                |row| row.get(0),
            )
            .expect("reads back");
        assert_eq!(canonical_text, "Text");
    }

    #[test]
    fn Test_Insert_Lineage_Should_Place_A_Heading_Anchored_Row()
    {
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::Lineage(crate::Lineage {
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
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Lineage(transaction, &bundle)).expect("inserts");

        let disposition: String = store
            .Connection()
            .query_row(
                "SELECT disposition FROM lineage WHERE source_heading_uid = 1",
                [],
                |row| row.get(0),
            )
            .expect("reads back");
        assert_eq!(disposition, "preserved-normalized");
    }

    #[test]
    fn Test_Insert_Omissions_Should_Place_A_Heading_Anchored_Row()
    {
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::Omission(crate::Omission {
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
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Omissions(transaction, &bundle)).expect("inserts");

        let reason: String = store
            .Connection()
            .query_row("SELECT reason FROM omissions WHERE source_heading_uid = 1", [], |row| {
                row.get(0)
            })
            .expect("reads back");
        assert_eq!(reason, "superseded");
    }

    #[test]
    fn Test_Insert_Record_Front_Matter_Should_Decode_The_Tags_Json_Column()
    {
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::RecordFrontMatter(crate::FrontMatter {
                document: crate::DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                node_id: "N1".to_owned(),
                status: "accepted".to_owned(),
                version: 1,
                tags: vec!["architecture".to_owned()],
            })],
        )
        .expect("builds");

        store
            .In_Transaction(|transaction| Insert_Record_Front_Matter(transaction, &bundle))
            .expect("inserts");

        let tags_json: String = store
            .Connection()
            .query_row("SELECT tags_json FROM record_front_matter WHERE document_uid = 1", [], |row| {
                row.get(0)
            })
            .expect("reads back");
        assert_eq!(tags_json, "[\"architecture\"]");
    }

    #[test]
    fn Test_Insert_Record_Relations_Should_Place_A_Declared_Relation_By_Document()
    {
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::RecordRelation(crate::RecordRelation {
                document: crate::DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                ordinal: 1,
                target: "N2".to_owned(),
                relation: "affects".to_owned(),
            })],
        )
        .expect("builds");

        store
            .In_Transaction(|transaction| Insert_Record_Relations(transaction, &bundle))
            .expect("inserts");

        let target: String = store
            .Connection()
            .query_row("SELECT target FROM record_relations WHERE document_uid = 1", [], |row| {
                row.get(0)
            })
            .expect("reads back");
        assert_eq!(target, "N2");
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
