use crate::BundleError;
use crate::bundle::Bundle;
use crate::model::{BlobEncoding, DocumentRef, OrdinalRef, Record};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use nomos_spec_model::ContentHash;
use nomos_spec_store::{SpecificationStore, Table};
use rusqlite::{Transaction, params};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportReport
{
    pub records: u32,
    pub counts: BTreeMap<String, u32>,
}

/// Rebuilds a store from a bundle.
///
/// # Errors
///
/// Returns [`BundleError::NotEmpty`] unless the store is empty, [`BundleError::Unresolved`]
/// if a record names something the bundle does not carry, and [`BundleError::Incomplete`]
/// if the store does not end up holding exactly what the bundle declared.
pub fn Import(store: &mut SpecificationStore, bundle: &Bundle) -> Result<ImportReport, BundleError>
{
    bundle.Verify_Counts()?;

    let supported = store.Version();
    let declared = bundle.Header().schema_version;
    if declared > supported
    {
        return Err(BundleError::TooNew {
            found: declared,
            supported,
        });
    }
    if declared < supported
    {
        return Err(BundleError::Malformed(format!(
            "the bundle is at schema version {declared} and this store is at {supported}; \
             bundle migration is not implemented, so importing it would guess"
        )));
    }

    Assert_Empty(store)?;

    return store.In_Transaction(|transaction| {
        // The relation-type vocabulary is self-referential (`inverse_of` names another
        // row in the same table), so no insertion order satisfies it. Deferring moves
        // every foreign-key check to commit without weakening any of them.
        transaction.pragma_update(None, "defer_foreign_keys", "ON")?;

        Insert_Blobs(transaction, bundle)?;
        Insert_Source_Documents(transaction, bundle)?;
        Insert_Source_Headings(transaction, bundle)?;
        Insert_Source_Blocks(transaction, bundle)?;
        Insert_Nodes(transaction, bundle)?;
        Insert_Node_Aliases(transaction, bundle)?;
        Insert_Node_History(transaction, bundle)?;
        Insert_Relation_Types(transaction, bundle)?;
        Insert_Relations(transaction, bundle)?;
        Insert_Normative_Statements(transaction, bundle)?;
        Insert_Lineage(transaction, bundle)?;
        Insert_Omissions(transaction, bundle)?;

        Assert_Landed(transaction, bundle)?;

        return Ok(ImportReport {
            records: bundle.Manifest().records,
            counts: bundle.Manifest().counts.clone(),
        });
    });
}

fn Assert_Empty(store: &SpecificationStore) -> Result<(), BundleError>
{
    for table in Table::All()
    {
        let rows = store.Count(*table)?;
        if rows > 0
        {
            return Err(BundleError::NotEmpty {
                table: table.Name().to_owned(),
                rows,
            });
        }
    }

    return Ok(());
}

/// The store holds exactly what the bundle said it would.
///
/// The mirror of the exporter's completeness guard: an insert that collapsed rows, or a
/// record kind nothing inserts, fails the import instead of producing a store that is
/// quietly smaller than its own bundle.
fn Assert_Landed(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for table in Table::All()
    {
        let sql = format!("SELECT count(*) FROM {}", table.Name());
        let landed: u32 = transaction.query_row(&sql, [], |row| row.get(0))?;
        let declared = bundle
            .Manifest()
            .counts
            .get(table.Name())
            .copied()
            .unwrap_or(0);

        if landed != declared
        {
            return Err(BundleError::Incomplete {
                table: table.Name().to_owned(),
                in_store: landed,
                exported: declared,
            });
        }
    }

    return Ok(());
}

fn Insert_Blobs(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::Blob(blob) = record
        else
        {
            continue;
        };

        let bytes = match blob.encoding
        {
            BlobEncoding::Utf8 => blob.content.clone().into_bytes(),
            BlobEncoding::Base64 => STANDARD
                .decode(&blob.content)
                .map_err(|error| BundleError::Malformed(format!("blob {}: {error}", blob.sha256)))?,
        };

        let digest = ContentHash::Of_Bytes(&bytes);
        if digest.As_Str() != blob.sha256
        {
            return Err(BundleError::Tampered {
                declared: blob.sha256.clone(),
                computed: digest.As_Str().to_owned(),
            });
        }
        if i64::try_from(bytes.len()).unwrap_or(i64::MAX) != blob.byte_length
        {
            return Err(BundleError::Malformed(format!(
                "blob {} declares {} byte(s) and carries {}",
                blob.sha256,
                blob.byte_length,
                bytes.len()
            )));
        }

        transaction.execute(
            "INSERT INTO blobs (sha256, byte_length, content) VALUES (?1, ?2, ?3)",
            params![blob.sha256, blob.byte_length, bytes],
        )?;
    }

    return Ok(());
}

fn Insert_Source_Documents(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::SourceDocument(document) = record
        else
        {
            continue;
        };

        let blob_uid = Blob_Uid(transaction, &document.blob_sha256)?;
        transaction.execute(
            "INSERT INTO source_documents (path, revision, blob_uid) VALUES (?1, ?2, ?3)",
            params![document.path, document.revision, blob_uid],
        )?;
    }

    return Ok(());
}

fn Insert_Source_Headings(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::SourceHeading(heading) = record
        else
        {
            continue;
        };

        let document_uid = Document_Uid(transaction, &heading.document)?;
        transaction.execute(
            "INSERT INTO source_headings (document_uid, ordinal, depth, title)
             VALUES (?1, ?2, ?3, ?4)",
            params![document_uid, heading.ordinal, heading.depth, heading.title],
        )?;
    }

    return Ok(());
}

fn Insert_Source_Blocks(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::SourceBlock(block) = record
        else
        {
            continue;
        };

        let document_uid = Document_Uid(transaction, &block.document)?;
        transaction.execute(
            "INSERT INTO source_blocks
             (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                document_uid,
                block.ordinal,
                block.kind,
                block.heading_path,
                block.text,
                block.content_hash,
                block.normalized_hash
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Nodes(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::Node(node) = record
        else
        {
            continue;
        };

        transaction.execute(
            "INSERT INTO nodes (node_id, kind, authority, representation, title, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                node.node_id,
                node.kind,
                node.authority,
                node.representation,
                node.title,
                node.deleted_at
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Node_Aliases(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::NodeAlias(alias) = record
        else
        {
            continue;
        };

        let node_uid = Node_Uid(transaction, &alias.node_id)?;
        transaction.execute(
            "INSERT INTO node_aliases (alias, node_uid) VALUES (?1, ?2)",
            params![alias.alias, node_uid],
        )?;
    }

    return Ok(());
}

fn Insert_Node_History(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::NodeHistory(entry) = record
        else
        {
            continue;
        };

        let node_uid = Node_Uid(transaction, &entry.node_id)?;
        transaction.execute(
            "INSERT INTO node_history
             (node_uid, ordinal, event, reason, previous_event_hash, event_hash, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                node_uid,
                entry.ordinal,
                entry.event,
                entry.reason,
                entry.previous_event_hash,
                entry.event_hash,
                entry.recorded_at
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Relation_Types(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::RelationType(relation_type) = record
        else
        {
            continue;
        };

        transaction.execute(
            "INSERT INTO relation_types (name, tier, inverse_of) VALUES (?1, ?2, ?3)",
            params![
                relation_type.name,
                relation_type.tier,
                relation_type.inverse_of
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Relations(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::Relation(relation) = record
        else
        {
            continue;
        };

        let from_uid = Node_Uid(transaction, &relation.from_node_id)?;
        let to_uid = Node_Uid(transaction, &relation.to_node_id)?;
        transaction.execute(
            "INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
             VALUES (?1, ?2, ?3)",
            params![from_uid, relation.relation_type, to_uid],
        )?;
    }

    return Ok(());
}

fn Insert_Normative_Statements(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::NormativeStatement(statement) = record
        else
        {
            continue;
        };

        let node_uid = Node_Uid(transaction, &statement.node_id)?;
        transaction.execute(
            "INSERT INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                node_uid,
                statement.statement_id,
                statement.kind,
                statement.canonical_text,
                statement.canonical_hash,
                statement.supersedes_hash
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Lineage(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::Lineage(lineage) = record
        else
        {
            continue;
        };

        let block_uid = Optional_Block_Uid(transaction, lineage.source_block.as_ref())?;
        let heading_uid = Optional_Heading_Uid(transaction, lineage.source_heading.as_ref())?;
        let node_uid = Optional_Node_Uid(transaction, lineage.target_node_id.as_deref())?;
        let statement_uid =
            Optional_Statement_Uid(transaction, lineage.target_statement_id.as_deref())?;

        transaction.execute(
            "INSERT INTO lineage
             (source_block_uid, source_heading_uid, disposition, target_node_uid, target_statement)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                block_uid,
                heading_uid,
                lineage.disposition,
                node_uid,
                statement_uid
            ],
        )?;
    }

    return Ok(());
}

fn Insert_Omissions(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    for record in bundle.Records()
    {
        let Record::Omission(omission) = record
        else
        {
            continue;
        };

        let block_uid = Optional_Block_Uid(transaction, omission.source_block.as_ref())?;
        let heading_uid = Optional_Heading_Uid(transaction, omission.source_heading.as_ref())?;

        transaction.execute(
            "INSERT INTO omissions
             (source_block_uid, source_heading_uid, reason, justification, decision_record)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                block_uid,
                heading_uid,
                omission.reason,
                omission.justification,
                omission.decision_record
            ],
        )?;
    }

    return Ok(());
}

/// A reference that does not resolve is an error, never a NULL. The distinction is the
/// whole point: a NULL here would leave a lineage row that counts as a row and points at
/// nothing.
fn Resolve(
    transaction: &Transaction<'_>,
    sql: &str,
    arguments: &[&dyn rusqlite::ToSql],
    record: &str,
    reference: String,
) -> Result<i64, BundleError>
{
    return transaction
        .query_row(sql, arguments, |row| row.get(0))
        .map_err(|error| match error
        {
            rusqlite::Error::QueryReturnedNoRows => BundleError::Unresolved {
                record: record.to_owned(),
                reference,
            },
            other => BundleError::Sql(other.to_string()),
        });
}

fn Blob_Uid(transaction: &Transaction<'_>, sha256: &str) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT uid FROM blobs WHERE sha256 = ?1",
        &[&sha256],
        "blob",
        sha256.to_owned(),
    );
}

fn Document_Uid(transaction: &Transaction<'_>, document: &DocumentRef) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
        &[&document.path, &document.revision],
        "source document",
        format!("{}@{}", document.path, document.revision),
    );
}

fn Node_Uid(transaction: &Transaction<'_>, node_id: &str) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT uid FROM nodes WHERE node_id = ?1",
        &[&node_id],
        "node",
        node_id.to_owned(),
    );
}

fn Optional_Node_Uid(
    transaction: &Transaction<'_>,
    node_id: Option<&str>,
) -> Result<Option<i64>, BundleError>
{
    return node_id.map(|id| Node_Uid(transaction, id)).transpose();
}

fn Optional_Statement_Uid(
    transaction: &Transaction<'_>,
    statement_id: Option<&str>,
) -> Result<Option<i64>, BundleError>
{
    return statement_id
        .map(|id| {
            return Resolve(
                transaction,
                "SELECT uid FROM normative_statements WHERE statement_id = ?1",
                &[&id],
                "normative statement",
                id.to_owned(),
            );
        })
        .transpose();
}

fn Optional_Block_Uid(
    transaction: &Transaction<'_>,
    reference: Option<&OrdinalRef>,
) -> Result<Option<i64>, BundleError>
{
    return reference
        .map(|reference| {
            return Resolve(
                transaction,
                "SELECT b.uid FROM source_blocks b
                 JOIN source_documents d ON d.uid = b.document_uid
                 WHERE d.path = ?1 AND d.revision = ?2 AND b.ordinal = ?3",
                &[
                    &reference.document.path,
                    &reference.document.revision,
                    &reference.ordinal,
                ],
                "source block",
                format!(
                    "{}@{}#{}",
                    reference.document.path, reference.document.revision, reference.ordinal
                ),
            );
        })
        .transpose();
}

fn Optional_Heading_Uid(
    transaction: &Transaction<'_>,
    reference: Option<&OrdinalRef>,
) -> Result<Option<i64>, BundleError>
{
    return reference
        .map(|reference| {
            return Resolve(
                transaction,
                "SELECT h.uid FROM source_headings h
                 JOIN source_documents d ON d.uid = h.document_uid
                 WHERE d.path = ?1 AND d.revision = ?2 AND h.ordinal = ?3",
                &[
                    &reference.document.path,
                    &reference.document.revision,
                    &reference.ordinal,
                ],
                "source heading",
                format!(
                    "{}@{}#{}",
                    reference.document.path, reference.document.revision, reference.ordinal
                ),
            );
        })
        .transpose();
}
