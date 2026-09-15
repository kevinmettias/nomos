//! Inserting the source corpus: the blobs, the documents, and everything addressed inside one.

use base64::Engine as _;
use rusqlite::{Transaction, params};

use crate::BundleError;
use crate::Blob;
use crate::Bundle;
use crate::Record;

use super::Insert_Each;
use super::reference::{
    Blob_Uid, Block_Uid, Document_Uid,
};

pub(super) fn Insert_Blobs(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO blobs (sha256, byte_length, content) VALUES (?1, ?2, ?3)",
        |insert, record| {
            let Record::Blob(blob) = record
            else
            {
                return Ok(());
            };
            let bytes = Decode_Blob_Bytes(blob)?;
            Assert_Declared(blob, &bytes)?;
            insert.execute(params![blob.sha256, blob.byte_length, bytes])?;

            return Ok(());
        },
    );
}

/// A blob's bytes, in whichever form the bundle carried them.
fn Decode_Blob_Bytes(blob: &Blob) -> Result<Vec<u8>, BundleError>
{
    use crate::Encoding as BlobEncoding;
    use base64::engine::general_purpose::STANDARD;

    return match blob.encoding
    {
        BlobEncoding::Utf8 => Ok(blob.content.clone().into_bytes()),
        BlobEncoding::Base64 => STANDARD.decode(&blob.content).map_err(|error| {
            return BundleError::Malformed(format!("blob {}: {error}", blob.sha256));
        }),
    };
}

/// The bytes are what the blob said they were.
///
/// Both the digest and the length are checked, because they fail differently: a digest that
/// disagrees is content that changed on the way here, and a length that disagrees is a
/// bundle that miscounted what it was carrying.
fn Assert_Declared(blob: &Blob, bytes: &[u8]) -> Result<(), BundleError>
{
    use nomos_spec_model::ContentHash;

    let digest = ContentHash::Of_Bytes(bytes);

    if digest.As_String_Slice() != blob.sha256
    {
        return Err(BundleError::Tampered {
            declared: blob.sha256.clone(),
            computed: digest.As_String_Slice().to_owned(),
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

    return Ok(());
}

pub(super) fn Insert_Source_Documents(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO source_documents (path, revision, blob_uid) VALUES (?1, ?2, ?3)",
        |insert, record| {
            let Record::SourceDocument(document) = record
            else
            {
                return Ok(());
            };

            let blob_uid = Blob_Uid(transaction, &document.blob_sha256)?;
            insert.execute(params![document.path, document.revision, blob_uid])?;

            return Ok(());
        },
    );
}

pub(super) fn Insert_Source_Headings(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO source_headings (document_uid, ordinal, depth, title)
         VALUES (?1, ?2, ?3, ?4)",
        |insert, record| {
            let Record::SourceHeading(heading) = record
            else
            {
                return Ok(());
            };

            let document_uid = Document_Uid(transaction, &heading.document)?;
            insert.execute(params![
                document_uid,
                heading.ordinal,
                heading.depth,
                heading.title
            ])?;

            return Ok(());
        },
    );
}

pub(super) fn Insert_Source_Blocks(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO source_blocks
         (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        |insert, record| {
            let Record::SourceBlock(block) = record
            else
            {
                return Ok(());
            };

            let document_uid = Document_Uid(transaction, &block.document)?;
            insert.execute(params![
                document_uid,
                block.ordinal,
                block.kind,
                block.heading_path,
                block.text,
                block.content_hash,
                block.normalized_hash
            ])?;

            return Ok(());
        },
    );
}

pub(super) fn Insert_Source_Table_Rows(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    return Insert_Each(
        transaction,
        bundle,
        "INSERT INTO source_table_rows
         (source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
          content_hash, normalized_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        |insert, record| {
            let Record::SourceTableRow(row) = record
            else
            {
                return Ok(());
            };

            let block_uid = Block_Uid(transaction, &row.block)?;
            let cells = serde_json::to_string(&row.cells)
                .map_err(|error| BundleError::Sql(error.to_string()))?;
            insert.execute(params![
                block_uid,
                row.ordinal,
                row.table_ordinal,
                row.kind,
                cells,
                row.text,
                row.content_hash,
                row.normalized_hash
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
    fn Test_Insert_Blobs_Should_Place_A_Blob_By_Its_Declared_Digest()
    {
        let mut store = Fixture();
        let digest = nomos_spec_model::ContentHash::Of_Bytes(b"bye").As_String_Slice().to_owned();
        let bundle = Bundle::New(
            1,
            vec![Record::Blob(Blob {
                sha256: digest.clone(),
                byte_length: 3,
                encoding: crate::Encoding::Utf8,
                content: "bye".to_owned(),
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Blobs(transaction, &bundle)).expect("inserts");

        let byte_length: i64 = store
            .Connection()
            .query_row("SELECT byte_length FROM blobs WHERE sha256 = ?1", [&digest], |row| {
                row.get(0)
            })
            .expect("reads back");
        assert_eq!(byte_length, 3);
    }

    #[test]
    fn Test_Insert_Source_Documents_Should_Place_A_Document_By_Its_Blob()
    {
        let mut store = Fixture();
        let bundle = Bundle::New(
            1,
            vec![Record::SourceDocument(crate::Document {
                path: "other.md".to_owned(),
                revision: "v2".to_owned(),
                blob_sha256: "sha256:aa".to_owned(),
            })],
        )
        .expect("builds");

        store.In_Transaction(|transaction| Insert_Source_Documents(transaction, &bundle)).expect("inserts");

        let revision: String = store
            .Connection()
            .query_row("SELECT revision FROM source_documents WHERE path = 'other.md'", [], |row| {
                row.get(0)
            })
            .expect("reads back");
        assert_eq!(revision, "v2");
    }

    #[test]
    fn Test_Insert_Source_Headings_Should_Place_A_Heading_By_Its_Document()
    {
        let title = Placed_Value(
            Record::SourceHeading(crate::Heading {
                document: crate::DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                ordinal: 2,
                depth: 1,
                title: "Section Two".to_owned(),
            }),
            Insert_Source_Headings,
            "SELECT title FROM source_headings WHERE document_uid = 1 AND ordinal = 2",
        );
        assert_eq!(title, "Section Two");
    }

    #[test]
    fn Test_Insert_Source_Blocks_Should_Place_A_Block_By_Its_Document()
    {
        let text = Placed_Value(
            Record::SourceBlock(crate::Block {
                document: crate::DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                ordinal: 2,
                kind: "paragraph".to_owned(),
                heading_path: "Intro".to_owned(),
                text: "World.".to_owned(),
                content_hash: "sha256:hc2".to_owned(),
                normalized_hash: "sha256:nh2".to_owned(),
            }),
            Insert_Source_Blocks,
            "SELECT text FROM source_blocks WHERE document_uid = 1 AND ordinal = 2",
        );
        assert_eq!(text, "World.");
    }

    #[test]
    fn Test_Insert_Source_Table_Rows_Should_Decode_The_Cells_Json_Column()
    {
        let cells_json = Placed_Value(
            Record::SourceTableRow(crate::TableRow {
                block: crate::OrdinalRef {
                    document: crate::DocumentRef {
                        path: "doc.md".to_owned(),
                        revision: "v1".to_owned(),
                    },
                    ordinal: 1,
                },
                ordinal: 1,
                table_ordinal: 1,
                kind: "content".to_owned(),
                cells: vec!["a".to_owned(), "b".to_owned()],
                text: "a | b".to_owned(),
                content_hash: "sha256:rc".to_owned(),
                normalized_hash: "sha256:rn".to_owned(),
            }),
            Insert_Source_Table_Rows,
            "SELECT cells_json FROM source_table_rows WHERE source_block_uid = 1",
        );
        assert_eq!(cells_json, "[\"a\",\"b\"]");
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
        let bundle = Bundle::New(1, vec![record])
            .expect("the bundle carries exactly the record the caller placed");

        store
            .In_Transaction(|transaction| insert(transaction, &bundle))
            .expect("the insert places the row the caller reads back");

        return store
            .Connection()
            .query_row(sql, [], |row| row.get(0))
            .expect("the column the caller named is readable from the row it placed");
    }

    /// One blob, the document read from it and one block inside it — one row of every
    /// table this file's inserts can resolve a reference against.
    fn Fixture() -> SpecificationStore
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO blobs (sha256, byte_length, content) VALUES ('sha256:aa', 2, x'6869');
                 INSERT INTO source_documents (path, revision, blob_uid) VALUES ('doc.md', 'v1', 1);
                 INSERT INTO source_blocks
                     (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
                     VALUES (1, 1, 'paragraph', 'Intro', 'Hello.', 'sha256:hc', 'sha256:nh');",
            )
            .expect("populates every table this file's inserts resolve against");

        return store;
    }
}
