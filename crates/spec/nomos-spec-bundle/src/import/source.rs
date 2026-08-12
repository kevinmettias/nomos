//! Inserting the source corpus: the blobs, the documents, and everything addressed inside one.


use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use nomos_spec_model::ContentHash;
use rusqlite::{Transaction, params};

use crate::BundleError;
use crate::row::blob::Blob;
use crate::row::blob::encoding::BlobEncoding;
use crate::bundle::Bundle;
use crate::row::record::Record;

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
            let bytes = Decoded(blob)?;
            Assert_Declared(blob, &bytes)?;
            insert.execute(params![blob.sha256, blob.byte_length, bytes])?;

            return Ok(());
        },
    );
}

/// A blob's bytes, in whichever form the bundle carried them.
fn Decoded(blob: &Blob) -> Result<Vec<u8>, BundleError>
{
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
    let digest = ContentHash::Of_Bytes(bytes);

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
