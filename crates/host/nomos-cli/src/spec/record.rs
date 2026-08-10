//! Finding one record and printing it, or saying why it could not be found.

use super::{Assembly, RecordRequest, Channels, ExitCode, Report_Store_Error, DocumentSource, Absent_Or, NodeSummary};

/// Phase 2's question: what did this record say?
pub(super) fn Record(
    assembly: &Assembly,
    request: &RecordRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let revision = request.revision.as_deref();
    let documents = match assembly.store.Documents_Behind(&request.id, revision)
    {
        Ok(documents) => documents,
        Err(error) => return Report_Store_Error(&error, channels.notes),
    };

    return match documents.as_slice()
    {
        [only] => Printed_Record(&request.id, only, channels),
        [] => Nothing_Behind(assembly, request, channels.notes),
        held => Ambiguous_Revision(&request.id, held, channels.notes),
    };
}

/// The one document behind an identifier, with a note saying which it was.
pub(super) fn Printed_Record(id: &str, only: &DocumentSource, channels: &mut Channels<'_>) -> ExitCode
{
    let _ = writeln!(
        channels.notes,
        "{id}: {} at revision {}, {}",
        only.path, only.revision, only.content_hash
    );
    let _ = write!(channels.output, "{}", only.text);

    return ExitCode::Ok;
}

/// One identifier held at several revisions, refused rather than concatenated.
///
/// Two revisions of one record are two answers to "what did this say", and printing both
/// under one heading is how a reader ends up quoting the wrong one.
pub(super) fn Ambiguous_Revision(
    id: &str,
    documents: &[DocumentSource],
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let held = documents
        .iter()
        .map(|document| return format!("{} ({})", document.revision, document.path))
        .collect::<Vec<String>>()
        .join(", ");

    let _ = writeln!(
        notes,
        "{id} is held at {} revisions: {held}.\n\
         Narrow it with --revision; printing them one after another would make the output a \
         document that never existed.",
        documents.len()
    );

    return ExitCode::NotFound;
}

/// What to say when a record read produced no document.
///
/// Three different things, because they are three different situations and only one of
/// them is the reader's mistake.
pub(super) fn Nothing_Behind(
    assembly: &Assembly,
    request: &RecordRequest,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let summary = match assembly.store.Node_Summary(&request.id)
    {
        Ok(summary) => summary,
        Err(error) => return Report_Store_Error(&error, notes),
    };

    match summary
    {
        Some(node) => Note_Unsourced_Node(request, &node, notes),
        None => drop(writeln!(
            notes,
            "no node in this store is identified {}.",
            request.id
        )),
    }

    return Absent_Or(assembly, ExitCode::NotFound, notes);
}

/// A node the store holds with no source document recorded against it.
pub(super) fn Note_Unsourced_Node(
    request: &RecordRequest,
    node: &NodeSummary,
    notes: &mut dyn std::io::Write,
)
{
    let wanted = request
        .revision
        .as_deref()
        .map_or_else(String::new, |label| return format!(" at revision {label}"));

    let _ = writeln!(
        notes,
        "{} is in the store as a {} node ({}, {}) titled {:?}, and no source document is \
         recorded against it{wanted}.",
        request.id, node.kind, node.authority, node.representation, node.title
    );
}
