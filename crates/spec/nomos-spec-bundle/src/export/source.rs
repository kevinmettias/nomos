//! Reading the source corpus out: the blobs, the documents, and everything addressed inside one.

use crate::BundleError;
use crate::DocumentRef;
use crate::Record;
use crate::SourceTableRow;
use base64::Engine as _;
use rusqlite::Connection;

use super::{Collect_Rows, Columns, Decode_Json_Column};

pub(super) fn Collect_Blobs(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement =
        connection.prepare("SELECT sha256, byte_length, content FROM blobs ORDER BY sha256")?;
    let rows = statement
        .query_map([], |row| {
            let mut columns = Columns::Of(row);
            let sha256: String = columns.Next()?;
            let byte_length: i64 = columns.Next()?;
            let content: Vec<u8> = columns.Next()?;
            return Ok((sha256, byte_length, content));
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for (sha256, byte_length, content) in rows
    {
        let blob = A_Blob(sha256, byte_length, content);

        records.push(blob);
    }

    return Ok(());
}

/// A blob's bytes as the bundle spells them: the text itself where it is UTF-8, and base64
/// where it is not.
fn A_Blob(sha256: String, byte_length: i64, content: Vec<u8>) -> Record
{
    use crate::Blob;
    use crate::BlobEncoding;
    use base64::engine::general_purpose::STANDARD;

    let (encoding, spelled) = match String::from_utf8(content)
    {
        Ok(text) => (BlobEncoding::Utf8, text),
        Err(error) => (BlobEncoding::Base64, STANDARD.encode(error.as_bytes())),
    };

    return Record::Blob(Blob {
        sha256,
        byte_length,
        encoding,
        content: spelled,
    });
}

pub(super) fn Source_Documents(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    use crate::SourceDocument;

    return Collect_Rows(
        connection,
        records,
        "SELECT d.path, d.revision, b.sha256
         FROM source_documents d JOIN blobs b ON b.uid = d.blob_uid
         ORDER BY d.path, d.revision",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::SourceDocument(SourceDocument {
                path: columns.Next()?,
                revision: columns.Next()?,
                blob_sha256: columns.Next()?,
            }));
        },
    );
}

pub(super) fn Source_Headings(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    use crate::SourceHeading;

    return Collect_Rows(
        connection,
        records,
        "SELECT d.path, d.revision, h.ordinal, h.depth, h.title
         FROM source_headings h JOIN source_documents d ON d.uid = h.document_uid
         ORDER BY d.path, d.revision, h.ordinal",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::SourceHeading(SourceHeading {
                document: DocumentRef {
                    path: columns.Next()?,
                    revision: columns.Next()?,
                },
                ordinal: columns.Next()?,
                depth: columns.Next()?,
                title: columns.Next()?,
            }));
        },
    );
}

pub(super) fn Source_Blocks(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    use crate::SourceBlock;

    return Collect_Rows(
        connection,
        records,
        "SELECT d.path, d.revision, b.ordinal, b.kind, b.heading_path, b.text,
                b.content_hash, b.normalized_hash
         FROM source_blocks b JOIN source_documents d ON d.uid = b.document_uid
         ORDER BY d.path, d.revision, b.ordinal",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::SourceBlock(SourceBlock {
                document: DocumentRef {
                    path: columns.Next()?,
                    revision: columns.Next()?,
                },
                ordinal: columns.Next()?,
                kind: columns.Next()?,
                heading_path: columns.Next()?,
                text: columns.Next()?,
                content_hash: columns.Next()?,
                normalized_hash: columns.Next()?,
            }));
        },
    );
}

const TABLE_ROWS: &str =
    "SELECT d.path, d.revision, b.ordinal, r.ordinal, r.table_ordinal, r.kind,
            r.cells_json, r.text, r.content_hash, r.normalized_hash
     FROM source_table_rows r
     JOIN source_blocks b ON b.uid = r.source_block_uid
     JOIN source_documents d ON d.uid = b.document_uid
     ORDER BY d.path, d.revision, b.ordinal, r.ordinal";

pub(super) fn Source_Table_Rows(connection: &Connection, records: &mut Vec<Record>)
    -> Result<(), BundleError>
{
    let mut statement = connection.prepare(TABLE_ROWS)?;
    let rows = statement
        .query_map([], Read_A_Table_Row)?
        .collect::<Result<Vec<_>, _>>()?;
    for (mut record, cells) in rows
    {
        record.cells = Decode_Json_Column(&cells)?;
        records.push(Record::SourceTableRow(record));
    }

    return Ok(());
}

/// One row and the JSON cells column that travels beside it, still undecoded.
fn Read_A_Table_Row(row: &rusqlite::Row<'_>) -> rusqlite::Result<(SourceTableRow, String)>
{
    let mut columns = Columns::Of(row);
    let block = Table_Row_Block(&mut columns)?;
    let (ordinal, table_ordinal, kind) = Table_Row_Identity(&mut columns)?;
    let cells: String = columns.Next()?;

    return Ok((
        SourceTableRow {
            block,
            ordinal,
            table_ordinal,
            kind,
            cells: Vec::new(),
            text: columns.Next()?,
            content_hash: columns.Next()?,
            normalized_hash: columns.Next()?,
        },
        cells,
    ));
}

/// The block a table row belongs to: the three columns the query names for it.
fn Table_Row_Block(columns: &mut Columns<'_, '_>) -> rusqlite::Result<crate::OrdinalRef>
{
    use crate::OrdinalRef;

    return Ok(OrdinalRef {
        document: DocumentRef {
            path: columns.Next()?,
            revision: columns.Next()?,
        },
        ordinal: columns.Next()?,
    });
}

/// A table row's own ordinal, its table ordinal and its kind — the three columns the query
/// names immediately after the block it belongs to.
fn Table_Row_Identity(columns: &mut Columns<'_, '_>) -> rusqlite::Result<(i64, i64, String)>
{
    let ordinal = columns.Next()?;
    let table_ordinal = columns.Next()?;
    let kind = columns.Next()?;

    return Ok((ordinal, table_ordinal, kind));
}
