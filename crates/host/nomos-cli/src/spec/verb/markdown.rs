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
