//! Rendering `nomos spec table`'s answer, or the refusal saying why it has none.
//!
//! Resolving the address and reading the rows is `nomos-spec-orchestration::Table`'s job
//! now. This module keeps only the writing and the `ExitCode` a rendering layer is
//! responsible for.

use crate::spec::{Assembly, TableRequest, Channels, ExitCode, DocumentSource, TableLine, RowCensus, Report_Store_Error, PathMatch, Absent_Or};
use nomos_spec_orchestration::{TableAnswer, TableRefusal};

/// Phase 2's other half: the real rows.
pub(in crate::spec) fn Table(assembly: &Assembly, request: &TableRequest, channels: &mut Channels<'_>) -> ExitCode
{
    return match nomos_spec_orchestration::Table(assembly, request)
    {
        Ok(answer) => Printed_Answer(request, &answer, channels),
        Err(TableRefusal::Store(error)) => Report_Store_Error(&error, channels.notes),
        Err(TableRefusal::NoSuchDocument) => No_Such_Document(assembly, &request.document, channels.notes),
        Err(TableRefusal::AmbiguousDocument { matched, tier }) =>
        {
            Several_Documents(&request.document, matched, tier, channels.notes)
        }
        Err(TableRefusal::NoRows { document, tier, census }) =>
        {
            let resolved = ResolvedDocument { document: &document, census, tier };
            Unselected(assembly, request, &resolved, channels.notes)
        }
    };
}

/// A document that was actually read: its path, how it was matched, and how many pipe
/// lines it carries -- the three coordinates [`Note_Document`] and [`Unselected`] both need
/// and only ever receive together, from either a resolved [`TableAnswer`] or a
/// [`TableRefusal::NoRows`].
pub(super) struct ResolvedDocument<'a>
{
    pub(super) document: &'a DocumentSource,
    pub(super) census: RowCensus,
    pub(super) tier: PathMatch,
}

/// The rows an address resolved to, with a note saying which document they came from.
pub(super) fn Printed_Answer(request: &TableRequest, answer: &TableAnswer, channels: &mut Channels<'_>) -> ExitCode
{
    let resolved = ResolvedDocument { document: &answer.document, census: answer.census, tier: answer.tier };
    Note_Document(&resolved, &request.document, channels.notes);

    return Printed(&answer.lines, request, channels);
}

/// A document that resolved and was read, and the request's own narrowing selected no rows.
pub(super) fn Unselected(
    assembly: &Assembly,
    request: &TableRequest,
    resolved: &ResolvedDocument<'_>,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    Note_Document(resolved, &request.document, notes);

    let _ = writeln!(
        notes,
        "{} carries no table row{}.",
        resolved.document.path,
        Narrowed(request.block, request.table)
    );

    // The document was read, so this is an answer rather than a shortfall — unless the
    // narrowing selected a table that is not there, which the census makes visible either
    // way.
    return Nothing_Selected(assembly, resolved.census.lines, notes);
}

/// Which document was read, how it was matched, and how many pipe lines it carries.
pub(super) fn Note_Document(resolved: &ResolvedDocument<'_>, asked: &str, notes: &mut dyn std::io::Write)
{
    if resolved.tier != PathMatch::Exact
    {
        let _ = writeln!(notes, "{asked} matched {} by {}", resolved.document.path, resolved.tier.Label());
    }

    let _ = writeln!(
        notes,
        "{} at revision {}: {} pipe line(s) in the whole document — {} header, {} content, \
         {} separator",
        resolved.document.path,
        resolved.document.revision,
        resolved.census.lines,
        resolved.census.header,
        resolved.census.content,
        resolved.census.separator
    );
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
