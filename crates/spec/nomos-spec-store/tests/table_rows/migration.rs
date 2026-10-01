//! A store written before the header kind existed must not keep answering the old way.
//!
//! Every fresh store types its rows on the way in, so migration 3's re-derivation never runs
//! there and would be untested by the whole suite otherwise. The version-2 fixture is the
//! only thing that reaches it, which is why the fixture and the assertion stay together.

/// The version whose shape typed a table's column titles as content rows.
const VERSION_BEFORE_HEADER_KINDS: u32 = 2;

/// The migration that re-derives those titles back into header rows.
const RE_DERIVE_HEADERS: u32 = 3;

/// A store written before the header kind existed must not keep answering the old way.
///
/// This drives migration 3 directly: build the version-2 shape, type the header as content
/// the way version 2 did, apply migration 3's own statements, and read back. Without it,
/// "how many data rows" would depend on when the store was built.
#[test]
fn Test_Migrating_A_Version_Two_Store_Should_Re_Derive_Its_Headers()
{
    let connection = rusqlite::Connection::open_in_memory()
        .expect("an in-memory connection is this fixture's blank slate");

    Apply_Migrations(&connection, 1..=VERSION_BEFORE_HEADER_KINDS);
    connection
        .execute_batch(A_VERSION_TWO_TABLE)
        .expect("the fixture above is written in the statements version 2 accepted");

    let before = Table_Row_Kinds(&connection);

    assert_eq!(before, vec!["content", "separator", "content"], "the fixture is not version 2");
    Apply_Migrations(&connection, RE_DERIVE_HEADERS..=RE_DERIVE_HEADERS);
    assert_eq!(
        Table_Row_Kinds(&connection),
        vec!["header", "separator", "content"],
        "the header still reads as a datum after the migration"
    );
}

/// Every migration in a version range, in order.
fn Apply_Migrations(connection: &rusqlite::Connection, versions: std::ops::RangeInclusive<u32>)
{
    let wanted = nomos_spec_store::MIGRATIONS
        .iter()
        .filter(|migration| return versions.contains(&migration.version));

    for migration in wanted
    {
        for statement in migration.statements
        {
            connection
                .execute_batch(statement)
                .expect("MIGRATIONS holds statements this crate's own schema accepts");
        }
    }
}

/// One table as a version-2 store held it: no header kind, so the column titles are content.
const A_VERSION_TWO_TABLE: &str =
    "INSERT INTO blobs (sha256, byte_length, content) VALUES ('sha256:aa', 1, x'61');
     INSERT INTO source_documents (path, revision, blob_uid)
     SELECT 'doc.md', 'v14.36', uid FROM blobs;
     INSERT INTO source_blocks
     (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
     SELECT uid, 1, 'prose', '', '| Model |', 'sha256:bb', 'sha256:cc'
     FROM source_documents;

     INSERT INTO source_table_rows
     (source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
      content_hash, normalized_hash)
     SELECT uid, 1, 1, 'content', '[\"Model\"]', '| Model |', 'sha256:01', 'sha256:01'
     FROM source_blocks;
     INSERT INTO source_table_rows
     (source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
      content_hash, normalized_hash)
     SELECT uid, 2, 1, 'separator', '[\"---\"]', '| --- |', 'sha256:02', 'sha256:02'
     FROM source_blocks;
     INSERT INTO source_table_rows
     (source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
      content_hash, normalized_hash)
     SELECT uid, 3, 1, 'content', '[\"A\"]', '| A |', 'sha256:03', 'sha256:03'
     FROM source_blocks;";

fn Table_Row_Kinds(connection: &rusqlite::Connection) -> Vec<String>
{
    return connection
        .prepare("SELECT kind FROM source_table_rows ORDER BY ordinal")
        .and_then(|mut statement| {
            return statement
                .query_map([], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("the projection names the one column source_table_rows holds");
}
