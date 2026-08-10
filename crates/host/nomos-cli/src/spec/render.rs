//! Building one projection and putting it where it was asked for.

use super::{Assembly, RenderRequest, Channels, ExitCode, Catalogue, Report_Project_Error, Build, Profile, Resolved, Output, Path, Placed, ProjectError, EmptySection, Empty_Section, SIDECAR_SUFFIX, Stamp};

/// Phase 4's renderers, run.
pub(super) fn Render(assembly: &Assembly, request: &RenderRequest, channels: &mut Channels<'_>) -> ExitCode
{
    let catalogue = match Catalogue::Shipped()
    {
        Ok(catalogue) => catalogue,
        Err(error) => return Report_Project_Error(&error, channels.notes),
    };

    let declared = match Declared(&catalogue, request, channels.notes)
    {
        Ok(declared) => declared,
        Err(code) => return code,
    };

    let built = match Build(&assembly.store, &declared)
    {
        Ok(built) => built,
        Err(error) => return Report_Build_Error(assembly, &error, channels.notes),
    };

    return Placed_Projection(&built, &declared.id, &request.into, channels);
}

/// The shipped profile a run names, resolved against the subject it was given.
///
/// Resolved before the store is touched. A profile that names a subject and a run that
/// does not supply one disagree about what is being built, and the disagreement is
/// answerable without reading a single row.
pub(super) fn Declared(
    catalogue: &Catalogue,
    request: &RenderRequest,
    notes: &mut dyn std::io::Write,
) -> Result<Profile, ExitCode>
{
    let declared = Resolved(catalogue, &request.profile, notes)?;

    return match declared.For(request.subject.as_deref())
    {
        Ok(resolved) => Ok(resolved),
        Err(error) => Err(Report_Project_Error(&error, notes)),
    };
}

/// Both halves of a built projection, written where the run asked for them.
pub(super) fn Placed_Projection(
    built: &Output,
    id: &str,
    into: &Path,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let body = into.join(&built.path);
    let sidecar = into.join(&built.sidecar_path);
    let sheet = match built.Sidecar()
    {
        Ok(sheet) => sheet,
        Err(error) => return Report_Project_Error(&error, channels.notes),
    };

    for (path, content) in [(&body, &built.body), (&sidecar, &sheet)]
    {
        if let Some(code) = Placed(path, content, channels.notes)
        {
            return code;
        }
    }

    Report_Render(id, &body, &sidecar, channels.output);
    Report_Stamp(&built.stamp, channels.output);

    return ExitCode::Ok;
}

/// A projection that could not be built.
///
/// The empty-section case is pulled out because it is the one a store without its corpus
/// reaches, and answering it with the projection machinery's own message would send the
/// reader to change a profile because of a variable that is not set.
pub(super) fn Report_Build_Error(
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
