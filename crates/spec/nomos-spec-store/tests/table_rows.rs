//! P3-ROWS. Table rows as typed subjects.
//!
//! The block stays the preservation authority — it holds the verbatim text and both v14
//! hashes, and P1-GATE proves those reproduce. Rows exist so a loss report can name what
//! went missing by identity rather than by count, which is the plan's own stated bar.
//!
//! Typing rows rather than discarding separators is what keeps `282` and `258` two
//! queries over one table. Neither is bent to match the other, and OD-SPEC-002 records
//! why the two numbers differ.

use nomos_spec_model::{Segment, SourceBlock, Table_Rows};
use nomos_spec_store::{AUTHORED, SpecificationStore, StoreError, Table};
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

fn Count_Rows(store: &SpecificationStore, kind: Option<&str>) -> u32
{
    let sql = match kind
    {
        Some(_) => "SELECT count(*) FROM source_table_rows WHERE kind = ?1",
        None => "SELECT count(*) FROM source_table_rows",
    };

    return match kind
    {
        Some(label) => store
            .Connection()
            .query_row(sql, rusqlite::params![label], |row| row.get(0)),
        None => store.Connection().query_row(sql, [], |row| row.get(0)),
    }
    .expect("queries");
}

#[test]
fn Test_A_Table_Block_Should_Store_Every_Pipe_Line()
{
    let store = Stored(TABLE).expect("stores");

    assert_eq!(Count_Rows(&store, None), 4, "one line of the table did not land");
    assert_eq!(Count_Rows(&store, Some("content")), 3);
    assert_eq!(Count_Rows(&store, Some("separator")), 1);
}

/// The two numbers the report has to carry, from one table.
#[test]
fn Test_The_Store_Should_Answer_Both_Counts()
{
    let store = Stored(TABLE).expect("stores");

    let pipe_lines = Count_Rows(&store, None);
    let content = Count_Rows(&store, Some("content"));

    assert_ne!(
        pipe_lines, content,
        "the fixture has no separator, so it cannot show the two counts differing"
    );
    assert_eq!(pipe_lines.saturating_sub(content), 1, "one separator, one table");
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

    assert_eq!(Count_Rows(&store, None), 0);
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
fn Test_The_Domain_Volumes_Should_Answer_282_And_258()
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

    let pipe_lines = Count_Rows(&store, None);
    let content = Count_Rows(&store, Some("content"));
    let separators = Count_Rows(&store, Some("separator"));

    assert_eq!(pipe_lines, 282, "pipe lines");
    assert_eq!(content, 258, "content rows");
    assert_eq!(separators, 24, "one delimiter per table, so this is the table count");
    assert_eq!(
        content.saturating_add(separators),
        pipe_lines,
        "the two counts do not reconcile, so one of them is measuring something else"
    );
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
