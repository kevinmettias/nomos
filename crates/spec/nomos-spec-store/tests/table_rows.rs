//! P3-ROWS. Table rows as typed subjects.
//!
//! The block stays the preservation authority — it holds the verbatim text and both v14
//! hashes, and P1-GATE proves those reproduce. Rows exist so a loss report can name what
//! went missing by identity rather than by count, which is the plan's own stated bar.
//!
//! Typing rows rather than discarding separators is what keeps `282` and `258` two
//! queries over one table. Neither is bent to match the other, and OD-SPEC-002 records
//! why the two numbers differ.
//!
//! P3-ROW-LINEAGE adds the third kind. A header is authored text that is not a datum, so
//! `282`, `258` and `234` are three queries rather than one number and two subtractions —
//! and a concept minted from a row traces to that row and not to the block around it.

use nomos_spec_model::{Segment, SourceBlock, Table_Rows};
use nomos_spec_store::{AUTHORED, NodeRow, RowCensus, RowScope, SpecificationStore, StoreError, Table};
use std::path::{Path, PathBuf};

const TABLE: &str = "# Canonical domain model\n\n\
                     | Model | Responsibility |\n\
                     | --- | --- |\n\
                     | WorkspaceContext | Repository, branch, configuration. |\n\
                     | MetricTradeoffProjection | Cost against benefit. |\n";

fn Stored(markdown: &str) -> Result<SpecificationStore, StoreError>
{
    let mut store = SpecificationStore::In_Memory()?;
    let document = store.Put_Source_Document("doc.md", AUTHORED, markdown)?;
    store.Put_Source_Blocks(document, &Segment(markdown))?;
    return Ok(store);
}

/// Discards the store so a refusal can be asserted on: the store is not `Debug`, and
/// making it so purely to write `expect_err` would widen a public API for a test.
fn Refusal(markdown: &str) -> StoreError
{
    return match Stored(markdown)
    {
        Ok(_) => panic!("expected a refusal, got a store"),
        Err(error) => error,
    };
}

fn Census(store: &SpecificationStore, scope: RowScope) -> RowCensus
{
    return store.Row_Census(scope).expect("takes a census");
}

#[test]
fn Test_A_Table_Block_Should_Store_Every_Pipe_Line()
{
    let store = Stored(TABLE).expect("stores");
    let census = Census(&store, RowScope::Everything);

    assert_eq!(census.lines, 4, "one line of the table did not land");
    assert_eq!(census.header, 1, "the column titles");
    assert_eq!(census.content, 2, "WorkspaceContext and MetricTradeoffProjection");
    assert_eq!(census.separator, 1);
}

/// The three numbers the report has to carry, from one table and no arithmetic.
#[test]
fn Test_The_Store_Should_Answer_Every_Count_Separately()
{
    let store = Stored(TABLE).expect("stores");
    let census = Census(&store, RowScope::Everything);

    assert_ne!(
        census.lines, census.non_separator,
        "the fixture has no separator, so it cannot show the counts differing"
    );
    assert_ne!(
        census.non_separator, census.content,
        "the fixture has no header, so it cannot show these two differing"
    );
    assert_eq!(census.non_separator, 3, "header and content together");
    assert_eq!(
        census
            .header
            .saturating_add(census.content)
            .saturating_add(census.separator),
        census.lines,
        "a pipe line landed in no kind, so one of the counts measures something else"
    );
}

/// A count is a query over a scope, so "how many rows does this one table have" is not a
/// number somebody arrived at by filtering a larger one by hand.
#[test]
fn Test_A_Census_Should_Narrow_To_One_Table()
{
    let markdown = "| a |\n| --- |\n| 1 |\n\nbetween\n\n| b |\n| --- |\n| 2 |\n| 3 |\n";
    let store = Stored(markdown).expect("stores");

    let everything = Census(&store, RowScope::Everything);
    assert_eq!(everything.content, 3);

    let block_uid: i64 = store
        .Connection()
        .query_row(
            "SELECT source_block_uid FROM source_table_rows
             WHERE cells_json LIKE '%\"3\"%' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("finds the second table's block");
    let table_ordinal: u32 = store
        .Connection()
        .query_row(
            "SELECT table_ordinal FROM source_table_rows
             WHERE cells_json LIKE '%\"3\"%' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("finds the second table");

    let scoped = Census(
        &store,
        RowScope::Table {
            block_uid,
            table_ordinal,
        },
    );

    assert_eq!(scoped.content, 2, "the scope leaked into the other table");
    assert_eq!(scoped.header, 1);
    assert_eq!(scoped.lines, 4);
}

/// A row must be addressable by what it says, or a loss report can only give a count.
#[test]
fn Test_A_Row_Should_Be_Findable_By_Its_Content()
{
    let store = Stored(TABLE).expect("stores");

    let found: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM source_table_rows
             WHERE kind = 'content' AND cells_json LIKE '%MetricTradeoffProjection%'",
            [],
            |row| row.get(0),
        )
        .expect("queries");

    assert_eq!(found, 1, "MetricTradeoffProjection is not addressable by identity");
}

#[test]
fn Test_A_Block_With_No_Table_Should_Store_No_Rows()
{
    let store = Stored("# Title\n\nJust prose.\n").expect("stores");

    assert_eq!(Census(&store, RowScope::Everything).lines, 0);
    assert!(store.Count(Table::SourceBlocks).expect("counts") > 0, "nothing was stored at all");
}

/// The invariant. Without it, typing a row is a way out of NSV-PRESERVE-002's view that
/// does not involve leaving the table.
#[test]
fn Test_A_Table_Without_A_Delimiter_Should_Be_Refused()
{
    let refusal = Refusal("| a | b |\n| 1 | 2 |\n");

    assert!(matches!(refusal, StoreError::Table { .. }), "{refusal}");
}

#[test]
fn Test_A_Table_With_Two_Delimiters_Should_Be_Refused()
{
    let refusal = Refusal("| a |\n| --- |\n| --- |\n| 1 |\n");

    assert!(matches!(refusal, StoreError::Table { .. }), "{refusal}");
}

/// A refused block must leave nothing behind.
#[test]
fn Test_A_Refused_Table_Should_Not_Write_Half_A_Document()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let markdown = "| a | b |\n| 1 | 2 |\n";
    let document = store
        .Put_Source_Document("doc.md", AUTHORED, markdown)
        .expect("stores the document");

    store
        .Put_Source_Blocks(document, &Segment(markdown))
        .expect_err("must refuse");

    assert_eq!(store.Count(Table::SourceBlocks).expect("counts"), 0);
    assert_eq!(store.Count(Table::SourceTableRows).expect("counts"), 0);
}

/// Re-ingest must not renumber rows: `uid` is what a lineage row would point at.
#[test]
fn Test_Rewriting_A_Document_Should_Not_Renumber_Its_Rows()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let document = store
        .Put_Source_Document("doc.md", AUTHORED, TABLE)
        .expect("stores");
    let blocks = Segment(TABLE);

    store.Put_Source_Blocks(document, &blocks).expect("writes");
    let before = Row_Uids(&store);
    store.Put_Source_Blocks(document, &blocks).expect("writes again");

    assert!(!before.is_empty(), "no rows, so this test proved nothing");
    assert_eq!(before, Row_Uids(&store), "the rows were reinserted under new uids");
}

fn Row_Uids(store: &SpecificationStore) -> Vec<i64>
{
    return store
        .Connection()
        .prepare("SELECT uid FROM source_table_rows ORDER BY source_block_uid, ordinal")
        .and_then(|mut statement| {
            return statement
                .query_map([], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("reads uids");
}

/// The measured half of the done-when, over the real corpus. Opt-in by path, and loud
/// rather than silent: a configured corpus that cannot be read fails.
#[test]
fn Test_The_Domain_Volumes_Should_Answer_282_258_And_234()
{
    let Some(root) = std::env::var_os("NOMOS_V14_CORPUS")
    else
    {
        return;
    };

    let root = PathBuf::from(root);
    assert!(
        root.is_dir(),
        "NOMOS_V14_CORPUS is set to {}, which is not a directory",
        root.display()
    );

    let mut store = SpecificationStore::In_Memory().expect("opens");
    let mut volumes = 0_u32;
    let mut with_tables = 0_u32;

    for path in Volumes(&root.join("01_authoring/domain_volumes"))
    {
        let markdown = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let name = path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("?");
        let blocks = Segment(&markdown);

        let document = store
            .Put_Source_Document(name, "v14.36", &markdown)
            .expect("stores the document");
        store
            .Put_Source_Blocks(document, &blocks)
            .unwrap_or_else(|error| panic!("{name}: {error}"));

        volumes = volumes.saturating_add(1);
        if blocks.iter().any(|block| !Table_Rows(block).is_empty())
        {
            with_tables = with_tables.saturating_add(1);
        }
    }

    assert_eq!(volumes, 10, "the ten domain volumes are the corpus this is measured over");
    assert_eq!(with_tables, 6, "six of the ten carry tables");

    let census = Census(&store, RowScope::Everything);

    // Each figure names what it counted. The plan's 282 is the pipe-line count; OD-SPEC-002
    // records that. 258 is every authored line, header included. 234 is the data.
    assert_eq!(census.lines, 282, "pipe lines over the ten domain volumes");
    assert_eq!(census.non_separator, 258, "authored lines, header rows included");
    assert_eq!(census.content, 234, "data rows");
    assert_eq!(census.header, 24, "one header per table");
    assert_eq!(census.separator, 24, "one delimiter per table, so this is the table count");
    assert_eq!(
        census
            .header
            .saturating_add(census.content)
            .saturating_add(census.separator),
        census.lines,
        "the counts do not reconcile, so one of them is measuring something else"
    );
    assert_eq!(
        census.header.saturating_add(census.content),
        census.non_separator,
        "non_separator disagrees with the two kinds it covers"
    );
}

/// Section 5's canonical domain model: 30 pipe lines, 28 data rows.
///
/// The count the plan states as 30 is the pipe-line count of the table that carries them,
/// and this is the query that tells the two apart. Naming the table by the content of its
/// first data row rather than by an ordinal, because an ordinal moves when the document
/// above it is edited and the row does not.
///
/// 28 is the rows. The rows name 38 models, because nine of them name more than one —
/// that count belongs to the restoration, which reads the cells, not to the census, which
/// counts lines.
#[test]
fn Test_The_Canonical_Domain_Model_Should_Answer_30_And_28()
{
    let Some(root) = std::env::var_os("NOMOS_V14_CORPUS")
    else
    {
        return;
    };

    let path = PathBuf::from(root).join(
        "01_authoring/domain_volumes/02-core-architecture-identity-and-configuration.md",
    );
    let markdown = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));

    let mut store = SpecificationStore::In_Memory().expect("opens");
    let document = store
        .Put_Source_Document("02-core.md", "v14.36", &markdown)
        .expect("stores the document");
    store
        .Put_Source_Blocks(document, &Segment(&markdown))
        .expect("stores the blocks");

    let (block_uid, table_ordinal): (i64, u32) = store
        .Connection()
        .query_row(
            "SELECT source_block_uid, table_ordinal FROM source_table_rows
             WHERE kind = 'content' AND cells_json LIKE '%WorkspaceContext%'",
            [],
            |row| return Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("the canonical domain model is no longer in this volume");

    let census = Census(
        &store,
        RowScope::Table {
            block_uid,
            table_ordinal,
        },
    );

    assert_eq!(census.lines, 30, "pipe lines");
    assert_eq!(census.content, 28, "data rows, which is not the model count");
    assert_eq!(census.header, 1);
    assert_eq!(census.separator, 1);
}

fn Volumes(directory: &Path) -> Vec<PathBuf>
{
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));

    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| return entry.path())
        .filter(|path| path.extension().and_then(std::ffi::OsStr::to_str) == Some("md"))
        .collect();
    paths.sort();

    return paths;
}

/// The property the restoration exists to establish: one canonical source, two
/// projections. A concept minted from a table row resolves back to that row.
#[test]
fn Test_A_Node_Restored_From_A_Row_Should_Trace_To_That_Row()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let document = store
        .Put_Source_Document("doc.md", "v14.36", TABLE)
        .expect("stores the document");
    store
        .Put_Source_Blocks(document, &Segment(TABLE))
        .expect("stores the blocks");

    let block_ordinal: u32 = store
        .Connection()
        .query_row(
            "SELECT b.ordinal FROM source_blocks b
             JOIN source_table_rows r ON r.source_block_uid = b.uid
             WHERE r.cells_json LIKE '%MetricTradeoffProjection%'",
            [],
            |row| row.get(0),
        )
        .expect("finds the block");
    let row_ordinal: u32 = store
        .Connection()
        .query_row(
            "SELECT ordinal FROM source_table_rows
             WHERE cells_json LIKE '%MetricTradeoffProjection%'",
            [],
            |row| row.get(0),
        )
        .expect("finds the row");

    let row_uid = store
        .Table_Row_Uid(document, block_ordinal, row_ordinal)
        .expect("queries")
        .expect("the row is addressable");
    let node = store
        .Upsert_Node(NodeRow {
            node_id: "CON-METRICTRADEOFF-001",
            kind: "concept",
            authority: "canonical",
            representation: "record",
            title: "MetricTradeoffProjection",
        })
        .expect("mints the concept");

    store
        .Put_Row_Lineage(row_uid, "preserved-verbatim", Some(node))
        .expect("records the lineage");

    let traced: String = store
        .Connection()
        .query_row(
            "SELECT r.text FROM lineage l
             JOIN source_table_rows r ON r.uid = l.source_table_row_uid
             JOIN nodes n ON n.uid = l.target_node_uid
             WHERE n.node_id = 'CON-METRICTRADEOFF-001'",
            [],
            |row| row.get(0),
        )
        .expect("the concept traces to no row");

    assert!(
        traced.contains("MetricTradeoffProjection"),
        "the concept traced to the wrong row: {traced}"
    );
    assert!(
        !traced.contains("WorkspaceContext"),
        "the concept traced to the whole table, not to its own row"
    );
}

/// Idempotence, over the column that is new. Re-running a restoration must not double
/// every concept's lineage, and the unique index is what makes it not.
#[test]
fn Test_Recording_A_Row_Lineage_Twice_Should_Write_One_Row()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let document = store
        .Put_Source_Document("doc.md", AUTHORED, TABLE)
        .expect("stores the document");
    store
        .Put_Source_Blocks(document, &Segment(TABLE))
        .expect("stores the blocks");

    let row_uid: i64 = store
        .Connection()
        .query_row(
            "SELECT uid FROM source_table_rows WHERE kind = 'content' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("finds a row");

    store
        .Put_Row_Lineage(row_uid, "preserved-verbatim", None)
        .expect("records");
    store
        .Put_Row_Lineage(row_uid, "preserved-verbatim", None)
        .expect("records again");

    assert_eq!(store.Count(Table::Lineage).expect("counts"), 1);
}

/// A lineage row must name something. The three-way CHECK is what stops the new column
/// from turning the constraint into "or nothing at all".
#[test]
fn Test_A_Lineage_Row_Naming_Nothing_Should_Be_Refused()
{
    let store = SpecificationStore::In_Memory().expect("opens");

    let targetless = store.Connection().execute(
        "INSERT INTO lineage (disposition) VALUES ('preserved-verbatim')",
        [],
    );

    assert!(targetless.is_err(), "a lineage row must name what it is about");
}

/// A store written before the header kind existed must not keep answering the old way.
///
/// Every fresh store types its rows on the way in, so migration 3's re-derivation never
/// runs there and would be untested by the whole suite otherwise. This drives it directly:
/// build the version-2 shape, type the header as content the way version 2 did, apply
/// migration 3's own statements, and read back. Without it, "how many data rows" would
/// depend on when the store was built.
#[test]
fn Test_Migrating_A_Version_Two_Store_Should_Re_Derive_Its_Headers()
{
    let connection = rusqlite::Connection::open_in_memory().expect("opens");

    for migration in nomos_spec_store::MIGRATIONS.iter().filter(|m| m.version <= 2)
    {
        for statement in migration.statements
        {
            connection.execute_batch(statement).expect("applies");
        }
    }

    connection
        .execute_batch(
            "INSERT INTO blobs (sha256, byte_length, content) VALUES ('sha256:aa', 1, x'61');
             INSERT INTO source_documents (path, revision, blob_uid)
             SELECT 'doc.md', 'v14.36', uid FROM blobs;
             INSERT INTO source_blocks
             (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
             SELECT uid, 1, 'prose', '', '| Model |', 'sha256:bb', 'sha256:cc'
             FROM source_documents;

             -- Version 2 had no header kind, so the column titles landed as content.
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
             FROM source_blocks;",
        )
        .expect("populates the version-2 shape");

    let before = Kinds(&connection);
    assert_eq!(before, vec!["content", "separator", "content"], "the fixture is not version 2");

    for migration in nomos_spec_store::MIGRATIONS.iter().filter(|m| m.version == 3)
    {
        for statement in migration.statements
        {
            connection.execute_batch(statement).expect("migrates");
        }
    }

    assert_eq!(
        Kinds(&connection),
        vec!["header", "separator", "content"],
        "the header still reads as a datum after the migration"
    );
}

fn Kinds(connection: &rusqlite::Connection) -> Vec<String>
{
    return connection
        .prepare("SELECT kind FROM source_table_rows ORDER BY ordinal")
        .and_then(|mut statement| {
            return statement
                .query_map([], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("reads kinds");
}

/// The separator carries no content, so normalization is what tells the two hashes apart
/// nowhere — but the row must still hash its own line, not the block's.
#[test]
fn Test_A_Row_Should_Hash_Its_Own_Line()
{
    let block = SourceBlock {
        ordinal: 1,
        kind: nomos_spec_model::BlockKind::Prose,
        heading_path: Vec::new(),
        text: "| a | b |\n| --- | --- |".to_owned(),
    };
    let rows = Table_Rows(&block);

    let first = rows.first().expect("a row");
    assert_eq!(first.Content_Hash(), nomos_spec_model::ContentHash::Of("| a | b |"));
    assert_ne!(
        first.Content_Hash(),
        block.Content_Hash(),
        "the row hashed the whole block"
    );
}
