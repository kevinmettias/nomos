//! Resolving a bundle's natural keys against the surrogates the schema assigned them.

// file-size: allow this file pairs its production code with its own inline #[cfg(test)]
// module; check-test-coverage keys a test's companion unit off the exact file it is
// textually written in, so these tests cannot move to a sibling file without losing
// their attribution to every function this file declares.
// responsibility: allow same reason -- the coupling that keeps this file whole is
// check-test-coverage's stem-based companion attribution, not a design choice.

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
            return Resolve_Uid_From_Sql_Arguments(
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
pub(super) fn Resolve_Uid_From_Sql_Arguments(
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
    return Resolve_Uid_From_Sql_Arguments(
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
    return Resolve_Uid_From_Sql_Arguments(
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
    return Resolve_Uid_From_Sql_Arguments(
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
            return Resolve_Uid_From_Sql_Arguments(
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
    return Resolve_Uid_From_Sql_Arguments(
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
            return Resolve_Uid_From_Sql_Arguments(
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
            return Resolve_Uid_From_Sql_Arguments(
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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_store::SpecificationStore;

    #[test]
    fn Test_Optional_Suite_Uid_Should_Return_None_When_No_Suite_Is_Given()
    {
        let mut store = Fixture();

        let found = store
            .In_Transaction(|transaction| Optional_Suite_Uid(transaction, Some("nomos")))
            .expect("resolves");
        let absent = store
            .In_Transaction(|transaction| Optional_Suite_Uid(transaction, None))
            .expect("resolves");

        assert_eq!(found, Some(1));
        assert_eq!(absent, None);
    }

    #[test]
    fn Test_Resolve_Uid_From_Sql_Arguments_Should_Report_An_Unresolved_Reference_By_Name()
    {
        let mut store = Fixture();

        let refusal = store
            .In_Transaction(|transaction| {
                Resolve_Uid_From_Sql_Arguments(
                    transaction,
                    "SELECT uid FROM nodes WHERE node_id = ?1",
                    &[&"GHOST"],
                    Referenced {
                        record: "node",
                        reference: "GHOST".to_owned(),
                    },
                )
            })
            .expect_err("a reference to nothing must be refused");

        assert!(
            matches!(refusal, BundleError::Unresolved { ref record, ref reference }
                if record == "node" && reference == "GHOST"),
            "{refusal}"
        );
    }

    #[test]
    fn Test_Blob_Uid_Should_Resolve_A_Blob_By_Its_Sha256()
    {
        let mut store = Fixture();

        let uid = store
            .In_Transaction(|transaction| Blob_Uid(transaction, "sha256:aa"))
            .expect("resolves");

        assert_eq!(uid, 1);
    }

    #[test]
    fn Test_Document_Uid_Should_Resolve_A_Document_By_Path_And_Revision()
    {
        let mut store = Fixture();

        let uid = store
            .In_Transaction(|transaction| Document_Uid(transaction, &A_Document_Ref()))
            .expect("resolves");

        assert_eq!(uid, 1);
    }

    #[test]
    fn Test_Node_Uid_Should_Resolve_A_Node_By_Its_Node_Id()
    {
        let mut store = Fixture();

        let uid = store.In_Transaction(|transaction| Node_Uid(transaction, "N1")).expect("resolves");

        assert_eq!(uid, 1);
    }

    #[test]
    fn Test_Optional_Node_Uid_Should_Return_None_When_No_Node_Id_Is_Given()
    {
        let mut store = Fixture();

        let found = store
            .In_Transaction(|transaction| Optional_Node_Uid(transaction, Some("N1")))
            .expect("resolves");
        let absent = store.In_Transaction(|transaction| Optional_Node_Uid(transaction, None)).expect("resolves");

        assert_eq!(found, Some(1));
        assert_eq!(absent, None);
    }

    #[test]
    fn Test_Optional_Statement_Uid_Should_Return_None_When_No_Statement_Id_Is_Given()
    {
        let mut store = Fixture();

        let found = store
            .In_Transaction(|transaction| Optional_Statement_Uid(transaction, Some("STMT-1")))
            .expect("resolves");
        let absent = store
            .In_Transaction(|transaction| Optional_Statement_Uid(transaction, None))
            .expect("resolves");

        assert_eq!(found, Some(1));
        assert_eq!(absent, None);
    }

    #[test]
    fn Test_Block_Uid_Should_Resolve_A_Source_Block_By_Its_Document_And_Ordinal()
    {
        let mut store = Fixture();

        let uid = store
            .In_Transaction(|transaction| Block_Uid(transaction, &An_Ordinal_Ref()))
            .expect("resolves");

        assert_eq!(uid, 1);
    }

    #[test]
    fn Test_Optional_Table_Row_Uid_Should_Return_None_When_No_Row_Is_Given()
    {
        let mut store = Fixture();
        let row = TableRowRef { block: An_Ordinal_Ref(), ordinal: 1 };

        let found = store
            .In_Transaction(|transaction| Optional_Table_Row_Uid(transaction, Some(&row)))
            .expect("resolves");
        let absent = store
            .In_Transaction(|transaction| Optional_Table_Row_Uid(transaction, None))
            .expect("resolves");

        assert_eq!(found, Some(1));
        assert_eq!(absent, None);
    }

    #[test]
    fn Test_Optional_Block_Uid_Should_Return_None_When_No_Block_Reference_Is_Given()
    {
        Assert_Optional_Ordinal_Lookup_Resolves(Optional_Block_Uid);
    }

    #[test]
    fn Test_Optional_Heading_Uid_Should_Return_None_When_No_Heading_Is_Given()
    {
        Assert_Optional_Ordinal_Lookup_Resolves(Optional_Heading_Uid);
    }

    #[test]
    fn Test_Submission_Uid_Should_Resolve_A_Submission_By_Its_Node_Id()
    {
        let mut store = Fixture();

        let uid = store.In_Transaction(|transaction| Submission_Uid(transaction, "N1")).expect("resolves");

        assert_eq!(uid, 1);
    }

    /// Both `Optional_Block_Uid` and `Optional_Heading_Uid` resolve `Some(reference)` to the
    /// one row [`Fixture`] seeds and `None` to nothing — the shape every test of either shares.
    fn Assert_Optional_Ordinal_Lookup_Resolves(
        lookup: impl Fn(&Transaction<'_>, Option<&OrdinalRef>) -> Result<Option<i64>, BundleError>,
    )
    {
        let mut store = Fixture();
        let reference = An_Ordinal_Ref();

        let found = store.In_Transaction(|transaction| lookup(transaction, Some(&reference))).expect("resolves");
        let absent = store.In_Transaction(|transaction| lookup(transaction, None)).expect("resolves");

        assert_eq!(found, Some(1));
        assert_eq!(absent, None);
    }

    /// The document [`Fixture`] seeds, named so a test asking for it does not respell it.
    fn A_Document_Ref() -> DocumentRef
    {
        return DocumentRef { path: "doc.md".to_owned(), revision: "v1".to_owned() };
    }

    /// The one block, heading and table row [`Fixture`] seeds all share this ordinal.
    fn An_Ordinal_Ref() -> OrdinalRef
    {
        return OrdinalRef { document: A_Document_Ref(), ordinal: 1 };
    }

    /// One row of everything this file resolves a surrogate for: a blob, the document read
    /// from it, one heading, one block, one table row inside that block, a suite, a node
    /// inside it, a normative statement and a submission, both filed under that node.
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
                 INSERT INTO source_blocks
                     (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
                     VALUES (1, 1, 'paragraph', 'Intro', 'Hello.', 'sha256:hc', 'sha256:nh');
                 INSERT INTO source_table_rows
                     (source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
                      content_hash, normalized_hash)
                     VALUES (1, 1, 1, 'content', '[\"a\"]', 'a', 'sha256:rc', 'sha256:rn');
                 INSERT INTO suites (suite_id, title, authority_root)
                     VALUES ('nomos', 'The Nomos Specification', 1);
                 INSERT INTO nodes
                     (node_id, kind, authority, representation, title, deleted_at, suite_uid)
                     VALUES ('N1', 'requirement', 'canonical', 'record', 'Node One', NULL, 1);
                 INSERT INTO normative_statements
                     (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
                     VALUES (1, 'STMT-1', 'Requirement', 'Text', 'sha256:aa', NULL);
                 INSERT INTO submissions
                     (node_uid, kind, form_contract_version, state, submitted_by, submitted_through)
                     VALUES (1, 'feature-request', 1, 'draft', 'me', 'test');",
            )
            .expect("populates every table this file resolves against");

        return store;
    }
}
