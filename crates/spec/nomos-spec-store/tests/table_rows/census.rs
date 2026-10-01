//! What a census counts, and that each of its figures is its own query.
//!
//! The point of the three numbers is that none of them is derived from another by
//! subtraction. So the fixtures here are chosen to make the figures differ: a table with no
//! separator cannot show `lines` and `non_separator` apart, and a suite whose fixtures
//! cannot separate two numbers is asserting they are equal.

use crate::stored::{
    Blame, RowScope, Row_Census_For_Scope, SpecificationStore, Store_Holding_Markdown, TABLE,
    Two_Columns_Of_Row,
};
use nomos_spec_store::Table;

/// The pipe lines the fixture table holds: one header, one delimiter and two data rows.
const TABLE_LINES: u32 = 4;

/// The fixture table's data rows, which name the two concepts this suite is about.
const TABLE_CONTENT_LINES: u32 = 2;

/// Every line of the fixture table that is not its delimiter: its header and its data rows.
const TABLE_NON_SEPARATOR_LINES: u32 = 3;

/// The data rows the two-table fixture holds across both of its tables together.
const TWO_TABLES_CONTENT_LINES: u32 = 3;

/// The data rows of that fixture's second table alone, which the scope must narrow to.
const SECOND_TABLE_CONTENT_LINES: u32 = 2;

/// The second table's own pipe lines: its header, its delimiter and its two data rows.
const SECOND_TABLE_LINES: u32 = 4;

#[test]
fn Test_A_Table_Block_Should_Store_Every_Pipe_Line()
{
    let store = Store_Holding_Markdown(TABLE).expect("TABLE is a well-formed table, so the store takes it");
    let census = Row_Census_For_Scope(&store, RowScope::Everything);

    assert_eq!(census.lines, TABLE_LINES, "one line of the table did not land");
    assert_eq!(census.header, 1, "the column titles");
    assert_eq!(
        census.content, TABLE_CONTENT_LINES,
        "WorkspaceContext and MetricTradeoffProjection"
    );
    assert_eq!(census.separator, 1);
}

/// The three numbers the report has to carry, from one table and no arithmetic.
#[test]
fn Test_The_Store_Should_Answer_Every_Count_Separately()
{
    let store =
        Store_Holding_Markdown(TABLE).expect("the same fixture body stores again for a second reading");
    let census = Row_Census_For_Scope(&store, RowScope::Everything);

    assert_ne!(
        census.lines, census.non_separator,
        "the fixture has no separator, so it cannot show the counts differing"
    );
    assert_ne!(
        census.non_separator, census.content,
        "the fixture has no header, so it cannot show these two differing"
    );
    assert_eq!(
        census.non_separator, TABLE_NON_SEPARATOR_LINES,
        "header and content together"
    );
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
    let store = Store_Holding_Markdown(markdown).expect("the two-table body is well formed, so both tables land");

    let everything = Row_Census_For_Scope(&store, RowScope::Everything);
    let second = The_Second_Table(&store);
    let scoped = Row_Census_For_Scope(&store, second);

    assert_eq!(everything.content, TWO_TABLES_CONTENT_LINES);
    assert_eq!(
        scoped.content, SECOND_TABLE_CONTENT_LINES,
        "the scope leaked into the other table"
    );
    assert_eq!(scoped.header, 1);
    assert_eq!(scoped.lines, SECOND_TABLE_LINES);
}

/// The second of the fixture's two tables, addressed by the only row that holds a `3`.
fn The_Second_Table(store: &SpecificationStore) -> RowScope
{
    let (block_uid, table_ordinal): (i64, u32) = Two_Columns_Of_Row(
        store,
        "SELECT source_block_uid, table_ordinal FROM source_table_rows
         WHERE cells_json LIKE '%\"3\"%' LIMIT 1",
        Blame("the only row whose cell is 3 is in the second table"),
    );

    return RowScope::Table {
        block_uid,
        table_ordinal,
    };
}

/// A row must be addressable by what it says, or a loss report can only give a count.
#[test]
fn Test_A_Row_Should_Be_Findable_By_Its_Content()
{
    let store = Store_Holding_Markdown(TABLE).expect("the fixture table is stored, so its rows can be searched");

    let found: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM source_table_rows
             WHERE kind = 'content' AND cells_json LIKE '%MetricTradeoffProjection%'",
            [],
            |row| row.get(0),
        )
        .expect("the count projects one column of a table the schema declares");

    assert_eq!(found, 1, "MetricTradeoffProjection is not addressable by identity");
}

#[test]
fn Test_A_Block_With_No_Table_Should_Store_No_Rows()
{
    let store =
        Store_Holding_Markdown("# Title\n\nJust prose.\n").expect("prose is a document the store accepts");

    assert_eq!(Row_Census_For_Scope(&store, RowScope::Everything).lines, 0);
    assert!(store.Count(Table::SourceBlocks).expect("counts") > 0, "nothing was stored at all");
}
