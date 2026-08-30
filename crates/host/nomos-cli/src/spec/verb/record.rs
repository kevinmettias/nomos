//! Rendering `nomos spec record`'s answer, or the refusal saying why it has none.
//!
//! The resolution itself -- which document answers, or why none does -- is
//! `nomos-spec-orchestration::Resolved_Record`'s job now. This module keeps only the writing and the
//! `ExitCode` a rendering layer is responsible for.

use crate::spec::{Assembly, RecordRequest, Channels, ExitCode, Report_Store_Error, DocumentSource, Absent_Or, NodeSummary};
use nomos_spec_orchestration::{RecordAnswer, RecordRefusal};

/// Phase 2's question: what did this record say?
pub(in crate::spec) fn Read_Record(
    assembly: &Assembly,
    request: &RecordRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    return match nomos_spec_orchestration::Resolved_Record(assembly, request)
    {
        Ok(answer) => Printed_Record(&answer, channels),
        Err(RecordRefusal::Store(error)) => Report_Store_Error(&error, channels.notes),
        Err(RecordRefusal::Ambiguous { id, documents }) =>
        {
            Ambiguous_Revision(&id, &documents, channels.notes)
        }
        Err(RecordRefusal::NotFound { id, revision, node }) =>
        {
            let absent = NotFound { id: &id, revision: revision.as_deref(), node: node.as_ref() };
            Nothing_Behind(assembly, &absent, channels.notes)
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

/// The three fields `RecordRefusal::NotFound` carries -- grouped so [`Nothing_Behind`]
/// takes one parameter instead of three that only ever travel together.
pub(super) struct NotFound<'a>
{
    pub(super) id: &'a str,
    pub(super) revision: Option<&'a str>,
    pub(super) node: Option<&'a NodeSummary>,
}

/// What to say when a record read produced no document.
///
/// Two different things, because they are two different situations and only one of them is
/// the reader's mistake.
pub(super) fn Nothing_Behind(assembly: &Assembly, absent: &NotFound<'_>, notes: &mut dyn std::io::Write) -> ExitCode
{
    match absent.node
    {
        Some(node) => Note_Unsourced_Node(absent.id, absent.revision, node, notes),
        None => drop(writeln!(notes, "no node in this store is identified {}.", absent.id)),
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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_orchestration::corpus::{Assemble_Corpus, CorpusRequest, DEFAULT_REVISION};

    /// A store with the embedded governing records seeded and no corpus, so
    /// [`Assembly::Is_Complete`] is deterministically `false` -- `root: None` records an
    /// absence unconditionally, regardless of what any real environment variable holds.
    fn Corpus_Unset_Assembly() -> Assembly
    {
        let request = CorpusRequest {
            variable: "NOMOS_SPEC_RECORD_TEST_CORPUS_UNSET".to_owned(),
            root: None,
            revision: DEFAULT_REVISION.to_owned(),
        };

        return Assemble_Corpus(&request).expect("the embedded governing records always seed");
    }

    fn Sample_Node() -> NodeSummary
    {
        return NodeSummary {
            node_id: "D-2".to_owned(),
            kind: "record".to_owned(),
            authority: "authored".to_owned(),
            representation: "markdown".to_owned(),
            title: "Some Title".to_owned(),
        };
    }

    #[test]
    fn Test_Read_Record_Should_Report_Absent_For_A_Record_The_Store_Never_Had()
    {
        let assembly = Corpus_Unset_Assembly();
        let request = RecordRequest { id: "P23-TESTING-HOST-NONEXISTENT-RECORD".to_owned(), revision: None };
        let mut output = Vec::new();
        let mut notes = Vec::new();
        let mut channels = Channels { output: &mut output, notes: &mut notes };

        let code = Read_Record(&assembly, &request, &mut channels);

        assert_eq!(code, ExitCode::Absent);
    }

    #[test]
    fn Test_Printed_Record_Should_Write_The_Documents_Text_To_Output_And_Note_Its_Origin()
    {
        let answer = RecordAnswer {
            id: "D-1".to_owned(),
            document: DocumentSource {
                uid: 1,
                path: "docs/records/d-1.md".to_owned(),
                revision: "authored".to_owned(),
                content_hash: "abc123".to_owned(),
                text: "# D-1\n\nbody".to_owned(),
            },
        };
        let mut output = Vec::new();
        let mut notes = Vec::new();
        let mut channels = Channels { output: &mut output, notes: &mut notes };

        let code = Printed_Record(&answer, &mut channels);

        assert_eq!(code, ExitCode::Ok);
        assert_eq!(String::from_utf8_lossy(&output), "# D-1\n\nbody");
        assert!(String::from_utf8_lossy(&notes).contains("abc123"));
    }

    #[test]
    fn Test_Ambiguous_Revision_Should_List_Every_Held_Revision()
    {
        let documents = vec![
            DocumentSource {
                uid: 1,
                path: "a.md".to_owned(),
                revision: "authored".to_owned(),
                content_hash: "h1".to_owned(),
                text: String::new(),
            },
            DocumentSource {
                uid: 2,
                path: "b.md".to_owned(),
                revision: "final".to_owned(),
                content_hash: "h2".to_owned(),
                text: String::new(),
            },
        ];
        let mut notes = Vec::new();

        let code = Ambiguous_Revision("D-1", &documents, &mut notes);

        assert_eq!(code, ExitCode::NotFound);
        let text = String::from_utf8_lossy(&notes);
        assert!(text.contains("authored"));
        assert!(text.contains("final"));
        assert!(text.contains("--revision"));
    }

    #[test]
    fn Test_Nothing_Behind_Should_Report_Absent_With_Or_Without_A_Sourceless_Node()
    {
        let assembly = Corpus_Unset_Assembly();

        let mut notes = Vec::new();
        let unknown = NotFound { id: "P23-NOPE", revision: None, node: None };
        assert_eq!(Nothing_Behind(&assembly, &unknown, &mut notes), ExitCode::Absent);
        assert!(String::from_utf8_lossy(&notes).contains("no node in this store is identified"));

        let node = Sample_Node();
        let mut notes = Vec::new();
        let sourceless = NotFound { id: "D-2", revision: None, node: Some(&node) };
        assert_eq!(Nothing_Behind(&assembly, &sourceless, &mut notes), ExitCode::Absent);
        assert!(String::from_utf8_lossy(&notes).contains("no source document is"));
    }

    #[test]
    fn Test_Note_Unsourced_Node_Should_Name_The_Node_And_Its_Revision()
    {
        let node = Sample_Node();
        let mut notes = Vec::new();

        Note_Unsourced_Node("D-2", Some("authored"), &node, &mut notes);

        let text = String::from_utf8_lossy(&notes);
        assert!(text.contains("D-2"));
        assert!(text.contains("at revision authored"));
        assert!(text.contains("record"));
    }
}
