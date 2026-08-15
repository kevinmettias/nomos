//! Resolving a bundle's natural keys against the surrogates the schema assigned them.

use rusqlite::{Transaction, params};

use crate::BundleError;
use crate::DocumentRef;
use crate::OrdinalRef;
use crate::TableRowRef;

pub(super) fn Optional_Suite_Uid(
    transaction: &Transaction<'_>,
    suite_id: Option<&str>,
) -> Result<Option<i64>, BundleError>
{
    return suite_id
        .map(|id| {
            return Resolve(
                transaction,
                "SELECT uid FROM suites WHERE suite_id = ?1",
                &[&id],
                Referenced {
                    record: "suite",
                    reference: id.to_owned(),
                },
            );
        })
        .transpose();
}

/// What a failed lookup names in its error: the kind of record that carried the reference,
/// and the reference exactly as it was written.
pub(super) struct Referenced<'a>
{
    record: &'a str,
    reference: String,
}

/// A reference that does not resolve is an error, never a NULL. The distinction is the
/// whole point: a NULL here would leave a lineage row that counts as a row and points at
/// nothing.
/// `sql` is `&'static str` so that the statement cannot be built at runtime, and
/// `prepare_cached` so that resolving N references parses and plans the lookup once rather
/// than once per reference — these resolvers are called from inside the insert loops.
pub(super) fn Resolve(
    transaction: &Transaction<'_>,
    sql: &'static str,
    // The lookups differ in both the number and the type of their keys — a path and a revision
    // arrive as `&String`, an ordinal as `&i64` — and a slice holds one type. Erasing each key
    // to `ToSql` is what lets one resolver serve all nine call sites; a generic parameter would
    // fix the key tuple and force a separate resolver per shape.
    arguments: &[&dyn rusqlite::ToSql],
    referenced: Referenced<'_>,
) -> Result<i64, BundleError>
{
    return transaction
        .prepare_cached(sql)
        .map_err(|error| return BundleError::Sql(error.to_string()))?
        .query_row(arguments, |row| row.get(0))
        .map_err(|error| match error
        {
            rusqlite::Error::QueryReturnedNoRows => BundleError::Unresolved {
                record: referenced.record.to_owned(),
                reference: referenced.reference,
            },
            other => BundleError::Sql(other.to_string()),
        });
}

pub(super) fn Blob_Uid(transaction: &Transaction<'_>, sha256: &str) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT uid FROM blobs WHERE sha256 = ?1",
        &[&sha256],
        Referenced {
            record: "blob",
            reference: sha256.to_owned(),
        },
    );
}

pub(super) fn Document_Uid(transaction: &Transaction<'_>, document: &DocumentRef) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
        &[&document.path, &document.revision],
        Referenced {
            record: "source document",
            reference: format!("{}@{}", document.path, document.revision),
        },
    );
}

pub(super) fn Node_Uid(transaction: &Transaction<'_>, node_id: &str) -> Result<i64, BundleError>
{
    return Resolve(
        transaction,
        "SELECT uid FROM nodes WHERE node_id = ?1",
        &[&node_id],
        Referenced {
            record: "node",
            reference: node_id.to_owned(),
        },
    );
}

pub(super) fn Optional_Node_Uid(
    transaction: &Transaction<'_>,
    node_id: Option<&str>,
) -> Result<Option<i64>, BundleError>
{
    return node_id.map(|id| Node_Uid(transaction, id)).transpose();
}

pub(super) fn Optional_Statement_Uid(
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
                Referenced {
                    record: "normative statement",
                    reference: id.to_owned(),
                },
            );
        })
        .transpose();
}

pub(super) fn Block_Uid(transaction: &Transaction<'_>, reference: &OrdinalRef) -> Result<i64, BundleError>
{
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
        Referenced {
            record: "source block",
            reference: format!(
                "{}@{}#{}",
                reference.document.path, reference.document.revision, reference.ordinal
            ),
        },
    );
}

pub(super) fn Optional_Table_Row_Uid(
    transaction: &Transaction<'_>,
    reference: Option<&TableRowRef>,
) -> Result<Option<i64>, BundleError>
{
    return reference
        .map(|reference| {
            return Resolve(
                transaction,
                "SELECT r.uid FROM source_table_rows r
                 JOIN source_blocks b ON b.uid = r.source_block_uid
                 JOIN source_documents d ON d.uid = b.document_uid
                 WHERE d.path = ?1 AND d.revision = ?2 AND b.ordinal = ?3 AND r.ordinal = ?4",
                &[
                    &reference.block.document.path,
                    &reference.block.document.revision,
                    &reference.block.ordinal,
                    &reference.ordinal,
                ],
                Referenced {
                    record: "source table row",
                    reference: format!(
                        "{}@{}#{}.{}",
                        reference.block.document.path,
                        reference.block.document.revision,
                        reference.block.ordinal,
                        reference.ordinal
                    ),
                },
            );
        })
        .transpose();
}

pub(super) fn Optional_Block_Uid(
    transaction: &Transaction<'_>,
    reference: Option<&OrdinalRef>,
) -> Result<Option<i64>, BundleError>
{
    return reference
        .map(|reference| return Block_Uid(transaction, reference))
        .transpose();
}

pub(super) fn Optional_Heading_Uid(
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
                Referenced {
                    record: "source heading",
                    reference: format!(
                        "{}@{}#{}",
                        reference.document.path, reference.document.revision, reference.ordinal
                    ),
                },
            );
        })
        .transpose();
}

/// The surrogate of the submission filed under `node_id`.
pub(super) fn Submission_Uid(transaction: &Transaction<'_>, node_id: &str) -> Result<i64, BundleError>
{
    return Ok(transaction.query_row(
        "SELECT s.uid FROM submissions s JOIN nodes n ON n.uid = s.node_uid
         WHERE n.node_id = ?1",
        params![node_id],
        |row| return row.get(0),
    )?);
}
