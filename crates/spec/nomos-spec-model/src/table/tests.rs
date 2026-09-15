//! The table reader, over markdown shaped the way a specification block writes it.
use super::*;
use crate::Segment;

fn Of_Kind(rows: &[TableRow], kind: RowKind) -> usize
{
    return rows.iter().filter(|row| row.kind == kind).count();
}

/// Where a table puts its first content row: the header is first, its delimiter second, and the
/// datum third. Every `rows.get` below names the row it is reaching for.
const FIRST_CONTENT_ROW: usize = 2;

/// A table with one content row is three pipe lines: the column titles, their delimiter, and
/// that one datum.
const ONE_ROW_TABLE_LINES: usize = 3;

#[test]
fn Test_A_Table_Should_Yield_Every_Pipe_Line_Typed()
{
    let rows = Rows_From_Markdown("| Model | Responsibility |\n| --- | --- |\n| WorkspaceContext | Repository. |\n");

    assert_eq!(rows.len(), ONE_ROW_TABLE_LINES);
    assert_eq!(rows.first().map(|row| row.kind), Some(RowKind::Header));
    assert_eq!(rows.get(1).map(|row| row.kind), Some(RowKind::Separator));
    assert_eq!(rows.get(FIRST_CONTENT_ROW).map(|row| row.kind), Some(RowKind::Content));
    assert_eq!(Of_Kind(&rows, RowKind::Content), 1, "the column titles are a datum");
}

/// A table with two content rows is four pipe lines: one header, one delimiter, two data rows.
const TWO_ROW_TABLE_LINES: usize = 4;

/// The two data rows that fixture carries, which is the content count the partition below reads
/// against the length it was given.
const TWO_ROW_TABLE_CONTENTS: usize = 2;

/// The three counts the loss report has to carry, from one table and no arithmetic.
#[test]
fn Test_The_Three_Counts_Should_Partition_The_Pipe_Lines()
{
    let rows = Rows_From_Markdown("| Model | Owns |\n| --- | --- |\n| A | one |\n| B | two |\n");

    assert_eq!(rows.len(), TWO_ROW_TABLE_LINES, "pipe lines");
    assert_eq!(Of_Kind(&rows, RowKind::Header), 1);
    assert_eq!(Of_Kind(&rows, RowKind::Content), TWO_ROW_TABLE_CONTENTS);
    assert_eq!(Of_Kind(&rows, RowKind::Separator), 1);
    assert_eq!(
        RowKind::All().iter().map(|kind| Of_Kind(&rows, *kind)).sum::<usize>(),
        rows.len(),
        "a pipe line landed in no kind, so one of the counts is measuring something else"
    );
}

/// A header spanning two lines is two header rows, which is what separates "every line before
/// the delimiter" from "the first one".
const TWO_LINE_HEADER_ROWS: usize = 2;

/// A header spanning two lines is two header rows, not one header and one datum.
#[test]
fn Test_Every_Line_Before_The_Delimiter_Should_Be_Header()
{
    let rows = Rows_From_Markdown("| Model | Owns |\n| (id) | (scope) |\n| --- | --- |\n| A | one |\n");

    assert_eq!(Of_Kind(&rows, RowKind::Header), TWO_LINE_HEADER_ROWS);
    assert_eq!(Of_Kind(&rows, RowKind::Content), 1);
}

/// A table carrying no delimiter has no line from which a header could have been typed, so both
/// of its lines are data.
const LINES_WITHOUT_A_DELIMITER: usize = 2;

/// A table with no delimiter has no known header, and the defect says so rather than
/// the typing guessing one.
#[test]
fn Test_A_Table_Without_A_Delimiter_Should_Type_Nothing_As_Header()
{
    let rows = Rows_From_Markdown("| a | b |\n| 1 | 2 |\n");

    assert_eq!(Of_Kind(&rows, RowKind::Header), 0);
    assert_eq!(Of_Kind(&rows, RowKind::Content), LINES_WITHOUT_A_DELIMITER);
    assert!(!Table_Defects(&rows).is_empty(), "the defect is what reports it");
}

/// A header per table in the block below, and one content row under each, which is what tells
/// "each table has its own header" from "the block has one".
const HEADERS_IN_THE_BLOCK: usize = 2;

/// The content row under each of those headers.
const CONTENTS_IN_THE_BLOCK: usize = 2;

/// Two tables in one block each get their own header.
#[test]
fn Test_Table_Rows_Should_Give_Each_Header_Its_Own_Table()
{
    let block = SourceBlock {
        ordinal: 1,
        kind: crate::BlockKind::Prose,
        heading_path: Vec::new(),
        text: "| a |\n| --- |\n| 1 |\nbetween\n| b |\n| --- |\n| 2 |".to_owned(),
    };
    let rows = Table_Rows(&block);

    assert_eq!(Of_Kind(&rows, RowKind::Header), HEADERS_IN_THE_BLOCK);
    assert_eq!(Of_Kind(&rows, RowKind::Content), CONTENTS_IN_THE_BLOCK);
}

#[test]
fn Test_Every_Kind_Should_Round_Trip_Through_Its_Label()
{
    for kind in RowKind::All()
    {
        assert_eq!(RowKind::Parse(kind.Label()), Some(*kind));
    }
}

/// Where [`RowKind::Separator`] sits in `RowKind::All()`. The other two arms are a zero and a
/// one, so this is the ordinal the mirror below has to say out loud.
const SEPARATOR_ORDINAL: usize = 2;

/// `RowKind::All()`'s own mirror, named in the doc comment above it.
///
/// The match has no wildcard arm. A variant added to `RowKind` without a matching arm
/// added here fails this file to *compile*, not merely to pass — the property D-134
/// asks a closed enum's mirror to have.
#[test]
fn Test_Every_RowKind_Should_Be_Matched_Exhaustively()
{
    fn Expected_Ordinal(kind: RowKind) -> usize
    {
        return match kind
        {
            RowKind::Header => 0,
            RowKind::Content => 1,
            RowKind::Separator => SEPARATOR_ORDINAL,
        };
    }

    for (index, kind) in RowKind::All().iter().enumerate()
    {
        assert_eq!(
            Expected_Ordinal(*kind),
            index,
            "{} is not matched at the position RowKind::All() puts it, so the exhaustive \
             match and the universe have drifted apart",
            kind.Label()
        );
    }
}

#[test]
fn Test_Cells_Should_Split_And_Trim()
{
    let rows = Rows_From_Markdown("| a | b |\n| --- | --- |\n| 1 | 2 |\n");

    assert_eq!(
        rows.first().map(|row| row.cells.clone()),
        Some(vec!["a".to_owned(), "b".to_owned()])
    );
}

/// An escaped pipe does not open a new column, so the row it sits in keeps its two cells. A
/// splitter that ignored the escape would answer three here.
const CELLS_IN_THE_ESCAPED_PIPE_ROW: usize = 2;

/// A cell may contain a pipe, and splitting on it would invent a column.
#[test]
fn Test_An_Escaped_Pipe_Should_Not_Split_A_Cell()
{
    let rows = Rows_From_Markdown("| Counted as | Value |\n| --- | --- |\n| Lines with \\| | 282 |\n");

    assert_eq!(
        rows.get(FIRST_CONTENT_ROW).map(|row| row.cells.len()),
        Some(CELLS_IN_THE_ESCAPED_PIPE_ROW)
    );
    assert_eq!(
        rows.get(FIRST_CONTENT_ROW).and_then(|row| row.cells.first().cloned()),
        Some("Lines with \\|".to_owned())
    );
}

#[test]
fn Test_A_Block_With_No_Table_Should_Yield_No_Rows()
{
    assert!(Rows_From_Markdown("Just a paragraph.\n\nAnd another.\n").is_empty());
}

#[test]
fn Test_The_Row_Text_Should_Be_The_Line_As_Authored()
{
    let rows = Rows_From_Markdown("| a | b |\n| --- | --- |\n");

    assert_eq!(rows.first().map(|row| row.text.as_str()), Some("| a | b |"));
}

#[test]
fn Test_Table_Defects_Should_Be_Empty_For_A_Well_Formed_Table()
{
    assert!(Table_Defects(&Rows_From_Markdown("| a |\n| --- |\n| 1 |\n")).is_empty());
}

/// Both lines of a table with no delimiter are rows the defect has to account for, and the
/// count it reports is what says so.
const NO_SEPARATOR_TABLE_ROWS: u32 = 2;

#[test]
fn Test_A_Table_Without_A_Delimiter_Should_Be_A_Defect()
{
    let defects = Table_Defects(&Rows_From_Markdown("| a |\n| 1 |\n"));

    assert!(
        matches!(defects.first(), Some(TableDefect::NoSeparator { rows: NO_SEPARATOR_TABLE_ROWS, .. })),
        "{defects:?}"
    );
}

/// The two delimiters the defect is raised for; one of them would have been a well-formed
/// table, so the count is the whole of the claim.
const TWO_DELIMITERS: u32 = 2;

#[test]
fn Test_A_Table_With_Two_Delimiters_Should_Be_A_Defect()
{
    let defects = Table_Defects(&Rows_From_Markdown("| a |\n| --- |\n| --- |\n| 1 |\n"));

    assert!(
        matches!(
            defects.first(),
            Some(TableDefect::ManySeparators { separators: TWO_DELIMITERS, .. })
        ),
        "{defects:?}"
    );
}

/// The two tables the block below carries, which is the ordinal the rows are numbered up to.
const TABLES_IN_THE_BLOCK: u32 = 2;

/// How many rows that block carries, which is the run of ordinals its rows are numbered with.
const ROWS_IN_THE_BLOCK: u32 = 4;

/// Two tables in one block are two tables, each judged on its own.
#[test]
fn Test_Tables_Should_Be_Numbered_Within_The_Block()
{
    let block = SourceBlock {
        ordinal: 1,
        kind: crate::BlockKind::Prose,
        heading_path: Vec::new(),
        text: "| a |\n| --- |\nbetween\n| b |\n| --- |".to_owned(),
    };
    let rows = Table_Rows(&block);
    let ordinals: Vec<u32> = (1..=ROWS_IN_THE_BLOCK).collect();

    assert_eq!(rows.iter().map(|row| row.table_ordinal).max(), Some(TABLES_IN_THE_BLOCK));
    assert!(Table_Defects(&rows).is_empty());
    assert_eq!(rows.iter().map(|row| row.ordinal).collect::<Vec<u32>>(), ordinals);
}

#[test]
fn Test_Alignment_Markers_Should_Still_Read_As_A_Delimiter()
{
    let rows = Rows_From_Markdown("| a | b |\n|:--- | ---:|\n| 1 | 2 |\n");

    assert_eq!(rows.get(1).map(|row| row.kind), Some(RowKind::Separator));
}

/// The v14 corpus's own shape: a box drawn inside a fence, whose sides are pipes.
#[test]
fn Test_Table_Rows_Should_Find_No_Table_In_A_Fenced_Block()
{
    let block = SourceBlock {
        ordinal: 1,
        kind: crate::BlockKind::Code,
        heading_path: Vec::new(),
        text: ASCII_ART.to_owned(),
    };

    assert!(Table_Rows(&block).is_empty(), "a fence holds literal text, not a table");
}

/// The same bytes outside a fence are still a table, so what decides is the block's kind
/// and not the shape of its lines. Without this, the test above would also pass against a
/// `Table_Rows` that had simply stopped recognizing tables.
#[test]
fn Test_Table_Rows_Should_Still_Read_The_Same_Lines_In_A_Prose_Block()
{
    let block = SourceBlock {
        ordinal: 1,
        kind: crate::BlockKind::Prose,
        heading_path: Vec::new(),
        text: ASCII_ART.to_owned(),
    };

    assert_eq!(
        Table_Rows(&block).len(),
        ASCII_ART_PIPE_LINES,
        "the same five pipe lines, read as a table because nothing says otherwise"
    );
}

/// Every pipe line in [`ASCII_ART`], which is what the test above counts and the fence test
/// above that finds none of.
const ASCII_ART_PIPE_LINES: usize = 5;

/// A fenced diagram of the shape the v14 game plan actually carries. The `+---+` borders
/// are not pipe lines and were never counted; the five between them were.
const ASCII_ART: &str = "+-------------------+\n\
                         | Request 1         |\n\
                         | Request 2         |\n\
                         | Request 3         |\n\
                         | Request 4         |\n\
                         | Request 5         |\n\
                         +-------------------+";

/// Every table every block of `markdown` carries, which is the one call each test above
/// derives its rows from.
fn Rows_From_Markdown(markdown: &str) -> Vec<TableRow>
{
    let blocks = Segment(markdown);
    return blocks.iter().flat_map(Table_Rows).collect();
}
