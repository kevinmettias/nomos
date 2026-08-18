//! Rendering `nomos spec record`'s answer, or the refusal saying why it has none.
//!
//! The resolution itself -- which document answers, or why none does -- is
//! `nomos-spec-orchestration::Record`'s job now. This module keeps only the writing and the
//! `ExitCode` a rendering layer is responsible for.

use crate::spec::{Assembly, RecordRequest, Channels, ExitCode, Report_Store_Error, DocumentSource, Absent_Or, NodeSummary};
use nomos_spec_orchestration::{RecordAnswer, RecordRefusal};

/// Phase 2's question: what did this record say?
pub(in crate::spec) fn Record(
    assembly: &Assembly,
    request: &RecordRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    return match nomos_spec_orchestration::Record(assembly, request)
    {
        Ok(answer) => Printed_Record(&answer, channels),
        Err(RecordRefusal::Store(error)) => Report_Store_Error(&error, channels.notes),
        Err(RecordRefusal::Ambiguous { id, documents }) =>
        {
            Ambiguous_Revision(&id, &documents, channels.notes)
        }
        Err(RecordRefusal::NotFound { id, revision, node }) =>
        {
            Nothing_Behind(assembly, &id, revision.as_deref(), node.as_ref(), channels.notes)
        }
    };
}

/// The one document behind an identifier, with a note saying which it was.
pub(super) fn Printed_Record(answer: &RecordAnswer, channels: &mut Channels<'_>) -> ExitCode
{
    let document = &answer.document;
    let _ = writeln!(
        channels.notes,
        "{}: {} at revision {}, {}",
        answer.id, document.path, document.revision, document.content_hash
    );
    let _ = write!(channels.output, "{}", document.text);

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
/// Two different things, because they are two different situations and only one of them is
/// the reader's mistake.
pub(super) fn Nothing_Behind(
    assembly: &Assembly,
    id: &str,
    revision: Option<&str>,
    node: Option<&NodeSummary>,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    match node
    {
        Some(node) => Note_Unsourced_Node(id, revision, node, notes),
        None => drop(writeln!(notes, "no node in this store is identified {id}.")),
    }

    return Absent_Or(assembly, ExitCode::NotFound, notes);
}

/// A node the store holds with no source document recorded against it.
pub(super) fn Note_Unsourced_Node(id: &str, revision: Option<&str>, node: &NodeSummary, notes: &mut dyn std::io::Write)
{
    let wanted = revision.map_or_else(String::new, |label| return format!(" at revision {label}"));

    let _ = writeln!(
        notes,
        "{id} is in the store as a {} node ({}, {}) titled {:?}, and no source document is \
         recorded against it{wanted}.",
        node.kind, node.authority, node.representation, node.title
    );
}
