use crate::block::SourceBlock;
use crate::normalize::ContentHash;
use serde::{Deserialize, Serialize};

/// What a line inside a table is.
///
/// A separator carries no authored content — it is the delimiter telling a reader where
/// the header stops. Typing rows rather than discarding separators keeps the line count
/// and the content count as two queries over one table, so neither number has to be bent
/// to match the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RowKind
{
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
            Self::Content => "content",
            Self::Separator => "separator",
        };
    }

    #[must_use]
    pub fn Parse(label: &str) -> Option<Self>
    {
        return match label
        {
            "content" => Some(Self::Content),
            "separator" => Some(Self::Separator),
            _ => None,
        };
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

        let cells = Split_Cells(trimmed);
        let kind = if Is_Separator(&cells) { RowKind::Separator } else { RowKind::Content };

        rows.push(TableRow {
            ordinal: u32::try_from(rows.len()).unwrap_or(u32::MAX).saturating_add(1),
            table_ordinal,
            kind,
            cells,
            text: line.to_owned(),
        });
    }

    return rows;
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
        let members = rows.iter().filter(|row| row.table_ordinal == table_ordinal);
        let mut total = 0_u32;
        let mut separators = 0_u32;
        for row in members
        {
            total = total.saturating_add(1);
            if row.kind == RowKind::Separator
            {
                separators = separators.saturating_add(1);
            }
        }

        if separators == 0
        {
            defects.push(TableDefect::NoSeparator {
                table_ordinal,
                rows: total,
            });
        }
        else if separators > 1
        {
            defects.push(TableDefect::ManySeparators {
                table_ordinal,
                separators,
            });
        }
    }

    return defects;
}

fn Is_Pipe_Line(trimmed: &str) -> bool
{
    return trimmed.len() >= 2 && trimmed.starts_with('|') && trimmed.ends_with('|');
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
        if escaped
        {
            current.push(character);
            escaped = false;
            continue;
        }
        match character
        {
            '\\' =>
            {
                escaped = true;
                current.push(character);
            }
            '|' =>
            {
                cells.push(current.trim().to_owned());
                current = String::new();
            }
            _ => current.push(character),
        }
    }
    cells.push(current.trim().to_owned());

    return cells;
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

fn Is_Dashes(cell: &str) -> bool
{
    let body = cell.strip_prefix(':').unwrap_or(cell);
    let body = body.strip_suffix(':').unwrap_or(body);

    return body.len() >= 2 && body.bytes().all(|byte| byte == b'-');
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

    #[test]
    fn Test_A_Table_Should_Yield_Every_Pipe_Line_Typed()
    {
        let rows = Rows("| Model | Responsibility |\n| --- | --- |\n| WorkspaceContext | Repository. |\n");

        assert_eq!(rows.len(), 3);
        assert_eq!(rows.first().map(|row| row.kind), Some(RowKind::Content));
        assert_eq!(rows.get(1).map(|row| row.kind), Some(RowKind::Separator));
        assert_eq!(rows.get(2).map(|row| row.kind), Some(RowKind::Content));
        assert_eq!(
            rows.iter().filter(|row| row.kind == RowKind::Content).count(),
            2,
            "the separator is counted as content"
        );
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
