//! Rendering `nomos spec render`'s answer, or the refusal saying why it has none.
//!
//! Building the profile and placing its two files is `nomos-spec-orchestration::Rendered_Projection`'s
//! job now, through `nomos-composer-std`'s `FILE_SYSTEM` -- the same composition-root choice
//! `nomos-cli::work` already makes for the ledger. This module keeps only the writing and
//! the `ExitCode` a rendering layer is responsible for.

use crate::spec::{Assembly, RenderRequest, Channels, ExitCode, Report_Project_Error, Report_Store_Error, Path, EmptySection, Empty_Section, SIDECAR_SUFFIX, Stamp};
use nomos_platform::FileSystemError;
use nomos_spec_orchestration::{RenderAnswer, RenderRefusal};
use nomos_spec_project::ProjectError;

/// Phase 4's renderers, run.
pub(in crate::spec) fn Render_Profile(assembly: &Assembly, request: &RenderRequest, channels: &mut Channels<'_>) -> ExitCode
{
    use nomos_composer_std::FILE_SYSTEM;

    return match nomos_spec_orchestration::Rendered_Projection(assembly, request, &FILE_SYSTEM)
    {
        Ok(answer) => Placed_Render(&answer, channels),
        Err(RenderRefusal::Store(error)) => Report_Store_Error(&error, channels.notes),
        Err(RenderRefusal::NoSuchProfile { requested, known }) => No_Such_Profile(&requested, &known, channels.notes),
        Err(RenderRefusal::Project(error)) => Report_Build_Error(assembly, &error, channels.notes),
        Err(RenderRefusal::Unwritable { path, error }) => Unwritable_Render(&path, &error, channels.notes),
    };
}

/// Both halves of a rendered output, already placed -- reported, and the stamp beside it.
fn Placed_Render(answer: &RenderAnswer, channels: &mut Channels<'_>) -> ExitCode
{
    Report_Render(&answer.id, &answer.body, &answer.sidecar, channels.output);
    Report_Stamp(&answer.stamp, channels.output);

    return ExitCode::Ok;
}

/// A built projection this build could not place where it was asked to go.
fn Unwritable_Render(path: &Path, error: &FileSystemError, notes: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "cannot write {}: {error}", path.display());

    return ExitCode::Unwritable;
}

/// An identifier the shipped catalogue does not carry.
pub(in crate::spec) fn No_Such_Profile(requested: &str, known: &[String], notes: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(
        notes,
        "no shipped profile is named {requested}. There are {}: {}",
        known.len(),
        known.join(", ")
    );

    return ExitCode::NotFound;
}

/// A projection that could not be built.
///
/// The empty-section case is pulled out because it is the one a store without its corpus
/// reaches, and answering it with the projection machinery's own message would send the
/// reader to change a profile because of a variable that is not set.
pub(in crate::spec) fn Report_Build_Error(
    assembly: &Assembly,
    error: &ProjectError,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let ProjectError::Empty {
        profile,
        section,
        content,
    } = error
    else
    {
        return Report_Project_Error(error, notes);
    };

    let empty = EmptySection {
        profile,
        section,
        content,
    };

    return Empty_Section(assembly, &empty, notes);
}

/// Where the two halves of a projection landed.
pub(super) fn Report_Render(id: &str, body: &Path, sidecar: &Path, output: &mut dyn std::io::Write)
{
    let _ = writeln!(
        output,
        "{id} -> {}\nsidecar ({SIDECAR_SUFFIX}) -> {}",
        body.display(),
        sidecar.display()
    );
}

/// What the projection selected, and what it hashes to.
pub(super) fn Report_Stamp(stamp: &Stamp, output: &mut dyn std::io::Write)
{
    for (title, count) in &stamp.sections
    {
        let _ = writeln!(output, "  {title}: {count}");
    }

    let _ = writeln!(
        output,
        "  content {} over inputs {}",
        stamp.content_digest, stamp.inputs_digest
    );
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_orchestration::corpus::{Assemble_Corpus, CorpusRequest, DEFAULT_REVISION};

    #[test]
    fn Test_Render_Profile_Should_Report_An_Unknown_Profile()
    {
        let assembly = Corpus_Unset_Assembly();
        let request = RenderRequest {
            profile: "no-such-profile-p23-testing-host".to_owned(),
            into: std::env::temp_dir(),
            subject: None,
        };
        let mut output = Vec::new();
        let mut notes = Vec::new();
        let mut channels = Channels { output: &mut output, notes: &mut notes };

        let code = Render_Profile(&assembly, &request, &mut channels);

        assert_eq!(code, ExitCode::NotFound);
    }

    #[test]
    fn Test_No_Such_Profile_Should_Name_The_Request_And_List_What_Is_Known()
    {
        let known = vec!["diagram-set".to_owned(), "github-markdown".to_owned()];
        let mut notes = Vec::new();

        let code = No_Such_Profile("bogus-profile", &known, &mut notes);

        assert_eq!(code, ExitCode::NotFound);
        let text = String::from_utf8_lossy(&notes);
        assert!(text.contains("bogus-profile"));
        assert!(text.contains("diagram-set"));
        assert!(text.contains("github-markdown"));
    }

    #[test]
    fn Test_Report_Build_Error_Should_Route_An_Empty_Section_Differently_From_Everything_Else()
    {
        let assembly = Corpus_Unset_Assembly();

        let mut notes = Vec::new();
        let empty = ProjectError::Empty {
            profile: "diagram-set".to_owned(),
            section: "records".to_owned(),
            content: "record",
        };
        assert_eq!(Report_Build_Error(&assembly, &empty, &mut notes), ExitCode::Absent);

        let mut notes = Vec::new();
        let other = ProjectError::Malformed("bad".to_owned());
        assert_eq!(Report_Build_Error(&assembly, &other, &mut notes), ExitCode::StoreError);
    }

    #[test]
    fn Test_Report_Render_Should_Print_Both_The_Body_And_Its_Sidecar_Path()
    {
        let mut output = Vec::new();

        Report_Render("diagram-set", Path::new("diagram.mmd"), Path::new("diagram.mmd.stamp"), &mut output);

        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("diagram-set"));
        assert!(text.contains("diagram.mmd"));
        assert!(text.contains("diagram.mmd.stamp"));
    }

    #[test]
    fn Test_Report_Stamp_Should_Print_Every_Section_Count_And_Both_Digests()
    {
        let stamp = Stamp {
            profile: "diagram-set".to_owned(),
            profile_digest: "profile-digest".to_owned(),
            format: nomos_spec_project::Format::Mermaid,
            output: "build/diagram.mmd".to_owned(),
            content_digest: "content-digest".to_owned(),
            inputs_digest: "inputs-digest".to_owned(),
            sections: vec![("records".to_owned(), 3)],
            inputs: Vec::new(),
        };
        let mut output = Vec::new();

        Report_Stamp(&stamp, &mut output);

        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("records: 3"));
        assert!(text.contains("content-digest"));
        assert!(text.contains("inputs-digest"));
    }

    /// A store with the embedded governing records seeded and no corpus, so
    /// [`Assembly::Is_Complete`] is deterministically `false` -- `root: None` records an
    /// absence unconditionally, regardless of what any real environment variable holds.
    fn Corpus_Unset_Assembly() -> Assembly
    {
        let request = CorpusRequest {
            variable: "NOMOS_SPEC_RENDER_TEST_CORPUS_UNSET".to_owned(),
            root: None,
            revision: DEFAULT_REVISION.to_owned(),
        };

        return Assemble_Corpus(&request).expect("the embedded governing records always seed");
    }
}
