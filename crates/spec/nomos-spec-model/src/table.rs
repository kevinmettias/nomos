//! Reading the table rows a block carries.

// A table's rows, what kind each row is, and the defects a table carries.
mod defect;
mod row;
mod row_kind;

pub use defect::TableDefect;
pub use row::TableRow;
pub use row_kind::RowKind;

use crate::SourceBlock;

/// Splits a block into the table rows it carries.
///
/// Returns empty for a block holding no pipe lines, which is most of them.
#[must_use]
pub fn Table_Rows(block: &SourceBlock) -> Vec<TableRow>
{
    let mut rows: Vec<TableRow> = Vec::new();
    let mut table_ordinal = 0_u32;
    let mut in_table = false;
    for line in block.text.split('\n')
    {
        let trimmed = line.trim();
        if !Is_Pipe_Line(trimmed)
        {
            in_table = false;
            continue;
        }
        if !in_table
        {
            in_table = true;
            table_ordinal = table_ordinal.saturating_add(1);
        }
        let row = One_Row(RawLine(line), Trimmed(trimmed), table_ordinal, rows.len());
        rows.push(row);
    }

    Retype_Headers(&mut rows);
    return rows;
}

/// A pipe line exactly as authored, kept distinct from [`Trimmed`] so the two cannot be
/// swapped at a call site: both are the same line, one carries whitespace the other has
/// already cut.
struct RawLine<'a>(&'a str);

/// A pipe line with its surrounding whitespace cut, kept distinct from [`RawLine`] for the
/// same reason.
struct Trimmed<'a>(&'a str);

/// One pipe line as a row, typed by whether its cells are a delimiter.
///
/// A header cannot be told from content here, because a line is not known to precede the
/// delimiter until the delimiter has been seen. [`Retype_Headers`] is that second pass.
fn One_Row(line: RawLine<'_>, trimmed: Trimmed<'_>, table_ordinal: u32, already: usize) -> TableRow
{
    let cells = Split_Cells(trimmed.0);
    let kind = if Is_Separator(&cells) { RowKind::Separator } else { RowKind::Content };

    return TableRow {
        ordinal: u32::try_from(already).unwrap_or(u32::MAX).saturating_add(1),
        table_ordinal,
        kind,
        cells,
        text: line.0.to_owned(),
    };
}

/// Everything a table places before its delimiter is header.
///
/// A second pass rather than a decision taken while reading, because a line cannot be
/// known to precede the delimiter until the delimiter has been seen. A table carrying no
/// delimiter keeps every line as content and is refused by [`Table_Defects`] — guessing
/// where its header stopped would be inventing the answer the defect exists to report.
fn Retype_Headers(rows: &mut [TableRow])
{
    let tables = rows.iter().map(|row| row.table_ordinal).max().unwrap_or(0);

    for table_ordinal in 1..=tables
    {
        let Some(delimiter) = rows
            .iter()
            .find(|row| row.table_ordinal == table_ordinal && row.kind == RowKind::Separator)
            .map(|row| row.ordinal)
        else
        {
            continue;
        };

        for row in rows
            .iter_mut()
            .filter(|row| row.table_ordinal == table_ordinal && row.ordinal < delimiter)
        {
            row.kind = RowKind::Header;
        }
    }
}

/// Exactly one delimiter per table.
///
/// Without this, typing a row is a way out of `NSV-PRESERVE-002`'s view that does not
/// involve leaving the table.
#[must_use]
pub fn Table_Defects(rows: &[TableRow]) -> Vec<TableDefect>
{
    let mut defects = Vec::new();
    let tables = rows.iter().map(|row| row.table_ordinal).max().unwrap_or(0);
    for table_ordinal in 1..=tables
    {
        let defect = Defect_Of(rows, table_ordinal);

        defects.extend(defect);
    }

    return defects;
}

/// One table's delimiter count, and what is wrong with it if anything is.
fn Defect_Of(rows: &[TableRow], table_ordinal: u32) -> Option<TableDefect>
{
    let mut total = 0_u32;
    let mut separators = 0_u32;
    for row in rows.iter().filter(|row| row.table_ordinal == table_ordinal)
    {
        total = total.saturating_add(1);
        separators = separators.saturating_add(u32::from(row.kind == RowKind::Separator));
    }
    if separators == 0
    {
        return Some(TableDefect::NoSeparator {
            table_ordinal,
            rows: total,
        });
    }
    if separators == 1
    {
        return None;
    }

    return Some(TableDefect::ManySeparators {
        table_ordinal,
        separators,
    });
}

/// The opening and closing pipes, which a line cannot be shorter than and still carry both.
const BOTH_PIPES: usize = 2;

fn Is_Pipe_Line(trimmed: &str) -> bool
{
    return trimmed.len() >= BOTH_PIPES && trimmed.starts_with('|') && trimmed.ends_with('|');
}

/// Splits on unescaped pipes, so a cell may contain a literal `\|`.
fn Split_Cells(trimmed: &str) -> Vec<String>
{
    let inner = trimmed.trim_start_matches('|').trim_end_matches('|');
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut escape = Escape::Plain;
    for character in inner.chars()
    {
        escape = Take_One_Character(character, escape, &mut cells, &mut current);
    }
    cells.push(current.trim().to_owned());

    return cells;
}

/// Whether the character just read was a backslash, so the one after it is literal rather
/// than a pipe delimiter.
///
/// A named two-state type in place of a bool, because a bare `true`/`false` at the call
/// site says nothing about which state either one means.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Escape
{
    Plain,
    Escaped,
}

/// One character folded into the cell being read, answering whether the next one is escaped.
fn Take_One_Character(
    character: char,
    escape: Escape,
    cells: &mut Vec<String>,
    current: &mut String,
) -> Escape
{
    if escape == Escape::Escaped
    {
        current.push(character);

        return Escape::Plain;
    }
    if character == '|'
    {
        cells.push(current.trim().to_owned());
        current.clear();

        return Escape::Plain;
    }

    current.push(character);

    return if character == '\\' { Escape::Escaped } else { Escape::Plain };
}

fn Is_Separator(cells: &[String]) -> bool
{
    let mut saw_one = false;
    for cell in cells
    {
        if cell.is_empty()
        {
            continue;
        }
        saw_one = true;
        if !Is_Dashes(cell)
        {
            return false;
        }
    }

    return saw_one;
}

/// The shortest run of dashes a separator cell is spelled with; one dash is a cell of prose.
const SHORTEST_DASH_RUN: usize = 2;

fn Is_Dashes(cell: &str) -> bool
{
    let body = cell.strip_prefix(':').unwrap_or(cell);
    let body = body.strip_suffix(':').unwrap_or(body);

    return body.len() >= SHORTEST_DASH_RUN && body.bytes().all(|byte| byte == b'-');
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Segment;

    fn Rows(markdown: &str) -> Vec<TableRow>
    {
        let blocks = Segment(markdown);
        return blocks.iter().flat_map(Table_Rows).collect();
    }

    fn Of_Kind(rows: &[TableRow], kind: RowKind) -> usize
    {
        return rows.iter().filter(|row| row.kind == kind).count();
    }

    #[test]
    fn Test_A_Table_Should_Yield_Every_Pipe_Line_Typed()
    {
        let rows = Rows("| Model | Responsibility |\n| --- | --- |\n| WorkspaceContext | Repository. |\n");

        assert_eq!(rows.len(), 3);
        assert_eq!(rows.first().map(|row| row.kind), Some(RowKind::Header));
        assert_eq!(rows.get(1).map(|row| row.kind), Some(RowKind::Separator));
        assert_eq!(rows.get(2).map(|row| row.kind), Some(RowKind::Content));
        assert_eq!(Of_Kind(&rows, RowKind::Content), 1, "the column titles are a datum");
    }

    /// The three counts the loss report has to carry, from one table and no arithmetic.
    #[test]
    fn Test_The_Three_Counts_Should_Partition_The_Pipe_Lines()
    {
        let rows = Rows("| Model | Owns |\n| --- | --- |\n| A | one |\n| B | two |\n");

        assert_eq!(rows.len(), 4, "pipe lines");
        assert_eq!(Of_Kind(&rows, RowKind::Header), 1);
        assert_eq!(Of_Kind(&rows, RowKind::Content), 2);
        assert_eq!(Of_Kind(&rows, RowKind::Separator), 1);
        assert_eq!(
            RowKind::All().iter().map(|kind| Of_Kind(&rows, *kind)).sum::<usize>(),
            rows.len(),
            "a pipe line landed in no kind, so one of the counts is measuring something else"
        );
    }

    /// A header spanning two lines is two header rows, not one header and one datum.
    #[test]
    fn Test_Every_Line_Before_The_Delimiter_Should_Be_Header()
    {
        let rows = Rows("| Model | Owns |\n| (id) | (scope) |\n| --- | --- |\n| A | one |\n");

        assert_eq!(Of_Kind(&rows, RowKind::Header), 2);
        assert_eq!(Of_Kind(&rows, RowKind::Content), 1);
    }

    /// A table with no delimiter has no known header, and the defect says so rather than
    /// the typing guessing one.
    #[test]
    fn Test_A_Table_Without_A_Delimiter_Should_Type_Nothing_As_Header()
    {
        let rows = Rows("| a | b |\n| 1 | 2 |\n");

        assert_eq!(Of_Kind(&rows, RowKind::Header), 0);
        assert_eq!(Of_Kind(&rows, RowKind::Content), 2);
        assert!(!Table_Defects(&rows).is_empty(), "the defect is what reports it");
    }

    /// Two tables in one block each get their own header.
    #[test]
    fn Test_A_Header_Should_Belong_To_Its_Own_Table()
    {
        let block = SourceBlock {
            ordinal: 1,
            kind: crate::block::BlockKind::Prose,
            heading_path: Vec::new(),
            text: "| a |\n| --- |\n| 1 |\nbetween\n| b |\n| --- |\n| 2 |".to_owned(),
        };
        let rows = Table_Rows(&block);

        assert_eq!(Of_Kind(&rows, RowKind::Header), 2);
        assert_eq!(Of_Kind(&rows, RowKind::Content), 2);
    }

    #[test]
    fn Test_Every_Kind_Should_Round_Trip_Through_Its_Label()
    {
        for kind in RowKind::All()
        {
            assert_eq!(RowKind::Parse(kind.Label()), Some(*kind));
        }
    }

    /// `RowKind::All()`'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to `RowKind` without a matching arm
    /// added here fails this file to *compile*, not merely to pass — the property D-134
    /// asks a closed enum's mirror to have.
    #[test]
    fn Test_Every_RowKind_Should_Be_Matched_Exhaustively()
    {
        fn Ordinal(kind: RowKind) -> usize
        {
            return match kind
            {
                RowKind::Header => 0,
                RowKind::Content => 1,
                RowKind::Separator => 2,
            };
        }

        for (index, kind) in RowKind::All().iter().enumerate()
        {
            assert_eq!(
                Ordinal(*kind),
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
        let rows = Rows("| a | b |\n| --- | --- |\n| 1 | 2 |\n");

        assert_eq!(
            rows.first().map(|row| row.cells.clone()),
            Some(vec!["a".to_owned(), "b".to_owned()])
        );
    }

    /// A cell may contain a pipe, and splitting on it would invent a column.
    #[test]
    fn Test_An_Escaped_Pipe_Should_Not_Split_A_Cell()
    {
        let rows = Rows("| Counted as | Value |\n| --- | --- |\n| Lines with \\| | 282 |\n");

        assert_eq!(rows.get(2).map(|row| row.cells.len()), Some(2));
        assert_eq!(
            rows.get(2).and_then(|row| row.cells.first().cloned()),
            Some("Lines with \\|".to_owned())
        );
    }

    #[test]
    fn Test_A_Block_With_No_Table_Should_Yield_No_Rows()
    {
        assert!(Rows("Just a paragraph.\n\nAnd another.\n").is_empty());
    }

    #[test]
    fn Test_The_Row_Text_Should_Be_The_Line_As_Authored()
    {
        let rows = Rows("| a | b |\n| --- | --- |\n");

        assert_eq!(rows.first().map(|row| row.text.as_str()), Some("| a | b |"));
    }

    #[test]
    fn Test_A_Well_Formed_Table_Should_Have_No_Defects()
    {
        assert!(Table_Defects(&Rows("| a |\n| --- |\n| 1 |\n")).is_empty());
    }

    #[test]
    fn Test_A_Table_Without_A_Delimiter_Should_Be_A_Defect()
    {
        let defects = Table_Defects(&Rows("| a |\n| 1 |\n"));

        assert!(
            matches!(defects.first(), Some(TableDefect::NoSeparator { rows: 2, .. })),
            "{defects:?}"
        );
    }

    #[test]
    fn Test_A_Table_With_Two_Delimiters_Should_Be_A_Defect()
    {
        let defects = Table_Defects(&Rows("| a |\n| --- |\n| --- |\n| 1 |\n"));

        assert!(
            matches!(
                defects.first(),
                Some(TableDefect::ManySeparators { separators: 2, .. })
            ),
            "{defects:?}"
        );
    }

    /// Two tables in one block are two tables, each judged on its own.
    #[test]
    fn Test_Tables_Should_Be_Numbered_Within_The_Block()
    {
        let block = SourceBlock {
            ordinal: 1,
            kind: crate::block::BlockKind::Prose,
            heading_path: Vec::new(),
            text: "| a |\n| --- |\nbetween\n| b |\n| --- |".to_owned(),
        };
        let rows = Table_Rows(&block);

        assert_eq!(rows.iter().map(|row| row.table_ordinal).max(), Some(2));
        assert!(Table_Defects(&rows).is_empty());
        assert_eq!(rows.iter().map(|row| row.ordinal).collect::<Vec<u32>>(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn Test_Alignment_Markers_Should_Still_Read_As_A_Delimiter()
    {
        let rows = Rows("| a | b |\n|:--- | ---:|\n| 1 | 2 |\n");

        assert_eq!(rows.get(1).map(|row| row.kind), Some(RowKind::Separator));
    }
}
