//! Reading the source corpus out: the blobs, the documents, and everything addressed inside one.

use crate::BundleError;
use crate::DocumentRef;
use crate::Record;
use crate::TableRow as SourceTableRow;
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
    use crate::Encoding as BlobEncoding;
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
    use crate::Document as SourceDocument;

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
    use crate::Heading as SourceHeading;

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
    use crate::Block as SourceBlock;

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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_store::SpecificationStore;

    /// The two bytes of `hi`, which is the content the fixture's blob carries.
    const FIXTURE_BLOB_BYTE_LENGTH: i64 = 2;

    #[test]
    fn Test_Collect_Blobs_Should_Spell_Utf8_Content_As_Text()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Collect_Blobs(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::Blob(crate::Blob {
                sha256: "sha256:aa".to_owned(),
                byte_length: FIXTURE_BLOB_BYTE_LENGTH,
                encoding: crate::Encoding::Utf8,
                content: "hi".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Source_Documents_Should_Read_A_Document_By_Its_Blob()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Source_Documents(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::SourceDocument(crate::Document {
                path: "doc.md".to_owned(),
                revision: "v1".to_owned(),
                blob_sha256: "sha256:aa".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Source_Headings_Should_Read_A_Heading_By_Its_Document()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Source_Headings(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::SourceHeading(crate::Heading {
                document: DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                ordinal: 1,
                depth: 1,
                title: "Intro".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Source_Blocks_Should_Read_A_Block_By_Its_Document()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Source_Blocks(store.Connection(), &mut records)
            .expect("the store's schema carries every table and column this collector's SQL names");

        assert_eq!(
            records,
            vec![Record::SourceBlock(crate::Block {
                document: DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                ordinal: 1,
                kind: "paragraph".to_owned(),
                heading_path: "Intro".to_owned(),
                text: "Hello.".to_owned(),
                content_hash: "sha256:hc".to_owned(),
                normalized_hash: "sha256:nh".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Source_Table_Rows_Should_Decode_The_Cells_Json_Column()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Source_Table_Rows(store.Connection(), &mut records)
            .expect("Source_Table_Rows ran over the store Fixture filled");

        assert_eq!(records, The_Row_It_Holds());
    }

    /// The one table row the fixture seeds, with its cells already encoded into the JSON
    /// column `Source_Table_Rows` has to decode.
    fn The_Row_It_Holds() -> Vec<Record>
    {
        return vec![Record::SourceTableRow(SourceTableRow {
            block: crate::OrdinalRef {
                document: DocumentRef {
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
        })];
    }

    /// One blob, the document read from it, one heading, one block and one table row
    /// inside that block — one row of everything this file reads.
    fn Fixture() -> SpecificationStore
    {
        let store = SpecificationStore::In_Memory()
            .expect("an in-memory store applies the schema this build carries");
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
                     VALUES (1, 1, 1, 'content', '[\"a\",\"b\"]', 'a | b', 'sha256:rc', 'sha256:rn');",
            )
            .expect("populates every table this file reads");

        return store;
    }
}
