//! Reading the table rows a block carries.

// A table's rows, what kind each row is, and the defects a table carries.
mod row_kind;
pub(crate) mod defect;
pub(crate) mod row;

pub use row_kind::RowKind;
use defect::Defect as TableDefect;
use row::Row as TableRow;

use crate::{BlockKind, SourceBlock};

/// Splits a block into the table rows it carries.
///
/// Returns empty for a block holding no pipe lines, which is most of them, and for a fenced
/// block whatever its lines look like.
///
/// A fence means the text inside it is literal, so there is no table in there to find --
/// every markdown reader agrees, and `Segment` already classifies a fenced block as
/// [`BlockKind::Code`], so the fact was available here and simply was not asked for. What
/// asking for it costs nothing and buys is real: the v14 corpus draws a box in a fence with
/// `+---+` borders and five `| Request N |` lines, and without this those five were a table
/// -- one with no delimiter row, which `Table_Defects` then refuses with "table 1 has 5
/// row(s) and no delimiter, so no row is a header". ASCII art is the common case, not a
/// contrived one; a fence is where people draw.
///
/// Asked here rather than at each caller, because all five of them -- two in
/// `nomos-spec-ingest`'s archaeology and restore, one in its revision census, and both of
/// `nomos-spec-store`'s block writer -- want the same answer, and a discriminator repeated
/// five times is one somebody eventually forgets.
#[must_use]
pub fn Table_Rows(block: &SourceBlock) -> Vec<TableRow>
{
    if block.kind == BlockKind::Code
    {
        return Vec::new();
    }

    let mut rows = Scan_Block(&block.text);
    Retype_Headers(&mut rows);

    return rows;
}

/// What one pass over a block's text carries from line to line: the rows collected so far,
/// whether the pass is inside a table, and which table that is.
struct ScanState
{
    rows: Vec<TableRow>,
    table_ordinal: u32,
    in_table: bool,
}

/// Every pipe line in `text` as a row, with each run of consecutive pipe lines numbered as
/// one table.
fn Scan_Block(text: &str) -> Vec<TableRow>
{
    let mut state = ScanState { rows: Vec::new(), table_ordinal: 0, in_table: false };
    for line in text.split('\n')
    {
        Absorb_Line(&mut state, line);
    }

    return state.rows;
}

/// Folds one line of a block into the scan: a pipe line becomes a row, and any other line
/// ends the table the scan was inside.
fn Absorb_Line(state: &mut ScanState, line: &str)
{
    let trimmed = line.trim();
    if !Is_Pipe_Line(trimmed)
    {
        state.in_table = false;

        return;
    }
    if !state.in_table
    {
        state.in_table = true;
        state.table_ordinal = state.table_ordinal.saturating_add(1);
    }

    let row = One_Row(RawLine(line), Trimmed(trimmed), state.table_ordinal, state.rows.len());
    state.rows.push(row);
}

/// The opening and closing pipes, which a line cannot be shorter than and still carry both.
const BOTH_PIPES: usize = 2;

fn Is_Pipe_Line(trimmed: &str) -> bool
{
    return trimmed.len() >= BOTH_PIPES && trimmed.starts_with('|') && trimmed.ends_with('|');
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
#[cfg(test)]
mod tests;
