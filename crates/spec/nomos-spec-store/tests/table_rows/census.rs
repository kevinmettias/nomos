//! What a census counts, and that each of its figures is its own query.
//!
//! The point of the three numbers is that none of them is derived from another by
//! subtraction. So the fixtures here are chosen to make the figures differ: a table with no
//! separator cannot show `lines` and `non_separator` apart, and a suite whose fixtures
//! cannot separate two numbers is asserting they are equal.

use crate::stored::{Census, RowScope, Stored, TABLE, Two};
use nomos_spec_store::Table;

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
    let (block_uid, table_ordinal): (i64, u32) = Two(
        &store,
        "SELECT source_block_uid, table_ordinal FROM source_table_rows
         WHERE cells_json LIKE '%\"3\"%' LIMIT 1",
        "finds the second table",
    );
    let scoped = Census(
        &store,
        RowScope::Table {
            block_uid,
            table_ordinal,
        },
    );

    assert_eq!(everything.content, 3);
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
