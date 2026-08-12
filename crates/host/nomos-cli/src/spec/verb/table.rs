//! Selecting one table out of one document and printing it.

use crate::spec::{Assembly, TableRequest, Channels, ExitCode, DocumentSource, TableLine, RowCensus, Report_Store_Error, Vanished, RowScope, PathMatch, Absent_Or};

/// Phase 2's other half: the real rows.
pub(in crate::spec) fn Table(assembly: &Assembly, request: &TableRequest, channels: &mut Channels<'_>) -> ExitCode
{
    let (uid, tier) = match Addressed(assembly, request, channels.notes)
    {
        Ok(addressed) => addressed,
        Err(code) => return code,
    };

    let read = match Read_Table(assembly, uid, request, channels.notes)
    {
        Ok(read) => read,
        Err(code) => return code,
    };

    Note_Document(&read, tier, &request.document, channels.notes);

    if read.lines.is_empty()
    {
        return Unselected(assembly, request, &read, channels.notes);
    }

    return Printed(&read.lines, request, channels);
}

/// A document, the rows the narrowing selected from it, and the census of the whole of it.
///
/// The census counts the document rather than the selection deliberately: it is what
/// distinguishes "this document has no tables" from "the block you named has none".
pub(super) struct ReadTable
{
    /// The document itself.
    found: DocumentSource,
    /// The rows under the current narrowing.
    lines: Vec<TableLine>,
    /// Every pipe line in the document, by kind.
    census: RowCensus,
}

/// Everything a `table` run reads, or the code saying which read failed.
pub(super) fn Read_Table(
    assembly: &Assembly,
    uid: i64,
    request: &TableRequest,
    notes: &mut dyn std::io::Write,
) -> Result<ReadTable, ExitCode>
{
    let found = match assembly.store.Document(uid)
    {
        Ok(Some(found)) => found,
        Ok(None) => return Err(Report_Store_Error(&Vanished(uid), notes)),
        Err(error) => return Err(Report_Store_Error(&error, notes)),
    };
    let lines = match assembly.store.Table_Lines(uid, request.block, request.table)
    {
        Ok(lines) => lines,
        Err(error) => return Err(Report_Store_Error(&error, notes)),
    };
    let census = match assembly.store.Row_Census(RowScope::Document(uid))
    {
        Ok(census) => census,
        Err(error) => return Err(Report_Store_Error(&error, notes)),
    };

    return Ok(ReadTable {
        found,
        lines,
        census,
    });
}

/// A document that was read and carried no row the narrowing selected.
pub(super) fn Unselected(
    assembly: &Assembly,
    request: &TableRequest,
    read: &ReadTable,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let _ = writeln!(
        notes,
        "{} carries no table row{}.",
        read.found.path,
        Narrowed(request.block, request.table)
    );

    // The document was read, so this is an answer rather than a shortfall — unless the
    // narrowing selected a table that is not there, which the census makes visible either
    // way.
    return Nothing_Selected(assembly, read.census.lines, notes);
}

/// Which document was read, how it was matched, and how many pipe lines it carries.
pub(super) fn Note_Document(read: &ReadTable, tier: PathMatch, asked: &str, notes: &mut dyn std::io::Write)
{
    let found = &read.found;
    let census = &read.census;

    if tier != PathMatch::Exact
    {
        let _ = writeln!(notes, "{asked} matched {} by {}", found.path, tier.Label());
    }

    let _ = writeln!(
        notes,
        "{} at revision {}: {} pipe line(s) in the whole document — {} header, {} content, \
         {} separator",
        found.path, found.revision, census.lines, census.header, census.content, census.separator
    );
}

/// The one document an address names, or the message saying why it names none or several.
pub(super) fn Addressed(
    assembly: &Assembly,
    request: &TableRequest,
    notes: &mut dyn std::io::Write,
) -> Result<(i64, PathMatch), ExitCode>
{
    let revision = request.revision.as_deref();
    let (matched, tier) = match assembly.store.Documents_Named(&request.document, revision)
    {
        Ok(found) => found,
        Err(error) => return Err(Report_Store_Error(&error, notes)),
    };

    return match matched.as_slice()
    {
        [uid] => Ok((*uid, tier)),
        [] => Err(No_Such_Document(assembly, &request.document, notes)),
        many => Err(Several_Documents(&request.document, many.len(), tier, notes)),
    };
}

/// An address that names nothing in this store.
pub(super) fn No_Such_Document(
    assembly: &Assembly,
    asked: &str,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let _ = writeln!(notes, "no document in this store is named {asked}.");

    return Absent_Or(assembly, ExitCode::NotFound, notes);
}

/// An address that names several, which is a question rather than an answer.
pub(super) fn Several_Documents(
    asked: &str,
    matched: usize,
    tier: PathMatch,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let _ = writeln!(
        notes,
        "{asked} matches {matched} documents by {}; give a whole path.",
        tier.Label()
    );

    return ExitCode::NotFound;
}

/// A document that was read and carried no row the narrowing selected.
pub(super) fn Nothing_Selected(
    assembly: &Assembly,
    pipe_lines: u32,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    if pipe_lines == 0 && !assembly.Is_Complete()
    {
        return Absent_Or(assembly, ExitCode::NotFound, notes);
    }

    return ExitCode::NotFound;
}

/// The rows themselves, and a note saying which blocks they came from.
pub(super) fn Printed(
    lines: &[TableLine],
    request: &TableRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    for line in lines
    {
        let _ = writeln!(channels.output, "{}", line.text);
    }

    let _ = writeln!(
        channels.notes,
        "printed {} row(s){} — block(s) {}",
        lines.len(),
        Narrowed(request.block, request.table),
        Ordinals(lines)
    );

    return ExitCode::Ok;
}

pub(super) fn Narrowed(block: Option<u32>, table: Option<u32>) -> String
{
    return match (block, table)
    {
        (None, None) => String::new(),
        (Some(block), None) => format!(" in block {block}"),
        (None, Some(table)) => format!(" in table {table} of any block"),
        (Some(block), Some(table)) => format!(" in table {table} of block {block}"),
    };
}

pub(super) fn Ordinals(lines: &[nomos_spec_store::TableLine]) -> String
{
    let mut seen: Vec<u32> = Vec::new();
    for line in lines
    {
        if !seen.contains(&line.block_ordinal)
        {
            seen.push(line.block_ordinal);
        }
    }

    return seen
        .iter()
        .map(u32::to_string)
        .collect::<Vec<String>>()
        .join(", ");
}
