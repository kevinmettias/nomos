//! Rendering `nomos spec render`'s answer, or the refusal saying why it has none.
//!
//! Building the profile and placing its two files is `nomos-spec-orchestration::Rendered_Projection`'s
//! job now, through `nomos-platform-std::StdFileSystem` -- the same composition-root choice
//! `nomos-cli::work` already makes for the ledger. This module keeps only the writing and
//! the `ExitCode` a rendering layer is responsible for.

use crate::spec::{Assembly, RenderRequest, Channels, ExitCode, Report_Project_Error, Report_Store_Error, Path, EmptySection, Empty_Section, SIDECAR_SUFFIX, Stamp};
use nomos_platform::FileSystemError;
use nomos_spec_orchestration::{RenderAnswer, RenderRefusal};
use nomos_spec_project::ProjectError;

/// Phase 4's renderers, run.
pub(in crate::spec) fn Render_Profile(assembly: &Assembly, request: &RenderRequest, channels: &mut Channels<'_>) -> ExitCode
{
    use nomos_platform_std::StdFileSystem;

    return match nomos_spec_orchestration::Rendered_Projection(assembly, request, &StdFileSystem)
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
