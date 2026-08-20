//! Inserting the graph: the nodes, what joins them, and what they were restored from.

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
    relation_type: &crate::RelationType,
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
