use crate::block::SourceBlock;
use crate::normalize::ContentHash;
use serde::{Deserialize, Serialize};

/// What a line inside a table is.
///
/// A separator carries no authored content — it is the delimiter telling a reader where
/// the header stops. A header names the columns; it is authored text, but it is not a
/// datum. Typing all three rather than discarding any of them keeps the line count, the
/// non-separator count and the data count as three queries over one table, so no number
/// has to be bent to match another.
///
/// Restoration is what forces the header to be its own kind. Minting a node per data row
/// of the canonical domain model, over a table whose header cannot be told from its data,
/// mints a concept named after the column titles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RowKind
{
    Header,
    Content,
    Separator,
}

impl RowKind
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Header => "header",
            Self::Content => "content",
            Self::Separator => "separator",
        };
    }

    #[must_use]
    pub fn Parse(label: &str) -> Option<Self>
    {
        return match label
        {
            "header" => Some(Self::Header),
            "content" => Some(Self::Content),
            "separator" => Some(Self::Separator),
            _ => None,
        };
    }

    /// Every kind, so a census cannot quietly omit one.
    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[Self::Header, Self::Content, Self::Separator];
    }
}

/// One line of a markdown table, held as a child of the block that carries it.
///
/// The block remains the preservation authority: it stores the verbatim text and both
/// v14 hashes, so byte completeness never depends on rows. Rows exist so a loss report
/// can name what went missing by identity rather than by count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableRow
{
    /// 1-based within the block.
    pub ordinal: u32,
    /// 1-based within the block. A block may carry more than one table.
    pub table_ordinal: u32,
    pub kind: RowKind,
    pub cells: Vec<String>,
    /// The line as authored.
    pub text: String,
}

impl TableRow
{
    #[must_use]
    pub fn Content_Hash(&self) -> ContentHash
    {
        return ContentHash::Of(&self.text);
    }

    #[must_use]
    pub fn Normalized_Hash(&self) -> ContentHash
    {
        return ContentHash::Of_Normalized(&self.text);
    }
}

/// A table that cannot be read as one.
///
/// Reported rather than repaired. A run of pipe lines with no delimiter is not a table,
/// and one with two delimiters has a second header nothing names — either way, guessing
/// which rows carry content is how a content row gets typed out of the preservation
/// rule's view without leaving the document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableDefect
{
    NoSeparator
    {
        table_ordinal: u32,
        rows: u32,
    },
    ManySeparators
    {
        table_ordinal: u32,
        separators: u32,
    },
}

impl core::fmt::Display for TableDefect
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::NoSeparator { table_ordinal, rows } => write!(
                formatter,
                "table {table_ordinal} has {rows} row(s) and no delimiter, so no row is a header"
            ),
            Self::ManySeparators {
                table_ordinal,
                separators,
            } => write!(
                formatter,
                "table {table_ordinal} has {separators} delimiters, so where its header stops is undecided"
            ),
        };
    }
}

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
        let row = One_Row(line, trimmed, table_ordinal, rows.len());
        rows.push(row);
    }

    Retype_Headers(&mut rows);
    return rows;
}

/// One pipe line as a row, typed by whether its cells are a delimiter.
///
/// A header cannot be told from content here, because a line is not known to precede the
/// delimiter until the delimiter has been seen. [`Retype_Headers`] is that second pass.
fn One_Row(line: &str, trimmed: &str, table_ordinal: u32, already: usize) -> TableRow
{
    let cells = Split_Cells(trimmed);
    let kind = if Is_Separator(&cells) { RowKind::Separator } else { RowKind::Content };

    return TableRow {
        ordinal: u32::try_from(already).unwrap_or(u32::MAX).saturating_add(1),
        table_ordinal,
        kind,
        cells,
        text: line.to_owned(),
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
    let mut escaped = false;
    for character in inner.chars()
    {
        escaped = Take_One_Character(character, escaped, &mut cells, &mut current);
    }
    cells.push(current.trim().to_owned());

    return cells;
}

/// One character folded into the cell being read, answering whether the next one is escaped.
fn Take_One_Character(
    character: char,
    escaped: bool,
    cells: &mut Vec<String>,
    current: &mut String,
) -> bool
{
    if escaped
    {
        current.push(character);

        return false;
    }
    if character == '|'
    {
        cells.push(current.trim().to_owned());
        current.clear();

        return false;
    }

    current.push(character);

    return character == '\\';
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
    use crate::block::Segment;

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
