//! Rendering one record as markdown, and saying whether it can be reproduced.

use crate::spec::{Assembly, RecordRequest, Channels, ExitCode, Report_Edit_Error, RecordProjection};

/// `D-129`'s round trip, read half: the record as the store's own rows render it.
///
/// Deliberately a second command rather than a flag on [`SpecCommand::Record`]. `record`
/// answers *what bytes went in*, which is a preservation question; this answers *what the
/// store can write back out*, which is an authoring one. A flag would make the two look like
/// two formats of one answer, and the whole point of `P9-AUTHORING` is that they were not the
/// same answer until now.
pub(in crate::spec) fn Render_Markdown(
    assembly: &Assembly,
    request: &RecordRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let projection = match nomos_spec_orchestration::Rendered_Markdown(assembly, request)
    {
        Ok(projection) => projection,
        Err(error) => return Report_Edit_Error(assembly, &error, channels.notes),
    };

    let _ = write!(channels.output, "{}", projection.markdown);
    let _ = writeln!(
        channels.notes,
        "{}: {} at revision {}, rendered from the store's rows as {}",
        request.id, projection.path, projection.revision, projection.projected_hash
    );

    return Reproducible_Projection(&projection, channels.notes);
}

/// Whether the store can write back the bytes it was given, said out loud when it cannot.
pub(super) fn Reproducible_Projection(projection: &RecordProjection, notes: &mut dyn std::io::Write) -> ExitCode
{
    if projection.Is_Matching_Source()
    {
        return ExitCode::Ok;
    }

    let _ = writeln!(
        notes,
        "the bytes this document was ingested from hash to {}, so the store cannot reproduce \
         them. A v14 record carrying a byte order mark is the ordinary reason (D-131), and an \
         edit through this surface is refused until that is settled rather than silently \
         normalised.",
        projection.source_hash
    );

    return ExitCode::Stale;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_orchestration::corpus::{Assemble_Corpus, CorpusRequest, DEFAULT_REVISION};

    #[test]
    fn Test_Render_Markdown_Should_Report_Absent_For_A_Record_The_Store_Never_Had()
    {
        let assembly = Corpus_Unset_Assembly();
        let request = RecordRequest { id: "P23-TESTING-HOST-NONEXISTENT-RECORD".to_owned(), revision: None };
        let mut output = Vec::new();
        let mut notes = Vec::new();
        let mut channels = Channels { output: &mut output, notes: &mut notes };

        let code = Render_Markdown(&assembly, &request, &mut channels);

        assert_eq!(code, ExitCode::Absent);
    }

    /// A store with the embedded governing records seeded and no corpus, so
    /// [`Assembly::Is_Complete`] is deterministically `false` -- `root: None` records an
    /// absence unconditionally, regardless of what any real environment variable holds.
    fn Corpus_Unset_Assembly() -> Assembly
    {
        let request = CorpusRequest {
            variable: "NOMOS_SPEC_MARKDOWN_TEST_CORPUS_UNSET".to_owned(),
            root: None,
            revision: DEFAULT_REVISION.to_owned(),
        };

        return Assemble_Corpus(&request).expect("the embedded governing records always seed");
    }

    #[test]
    fn Test_Reproducible_Projection_Should_Turn_A_Hash_Mismatch_Into_Stale()
    {
        let matching = RecordProjection {
            node_id: "D-1".to_owned(),
            path: "docs/records/d-1.md".to_owned(),
            revision: "authored".to_owned(),
            markdown: "text".to_owned(),
            source_hash: "abc".to_owned(),
            projected_hash: "abc".to_owned(),
        };
        let mut notes = Vec::new();
        assert_eq!(Reproducible_Projection(&matching, &mut notes), ExitCode::Ok);
        assert!(notes.is_empty());

        let mismatched = RecordProjection { projected_hash: "def".to_owned(), ..matching };
        let mut notes = Vec::new();
        assert_eq!(Reproducible_Projection(&mismatched, &mut notes), ExitCode::Stale);
        assert!(!notes.is_empty());
    }
}
