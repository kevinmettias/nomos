//! Previewing an edit and committing one.

use super::{Assembly, EditRequest, Channels, ExitCode, EPHEMERAL, CommitRequest, EditPreview, CommitReport, Path, EditError, Absent_Or};

/// The preview, printed, changing nothing.
pub(super) fn Preview(assembly: &Assembly, request: &EditRequest, channels: &mut Channels<'_>) -> ExitCode
{
    let staged = match Staged_Text(&request.from, channels.notes)
    {
        Ok(text) => text,
        Err(code) => return code,
    };

    let preview = match Previewed(assembly, request, &staged, channels.notes)
    {
        Ok(preview) => preview,
        Err(code) => return code,
    };

    let _ = writeln!(channels.output, "{}", preview.Describe());
    let _ = writeln!(channels.notes, "nothing was written. {EPHEMERAL}");

    return ExitCode::Ok;
}

/// The preview and then the commit, in that order, because the other order is not available.
pub(super) fn Commit(
    assembly: &mut Assembly,
    request: &CommitRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let text = match Staged_Text(&request.edit.from, channels.notes)
    {
        Ok(text) => text,
        Err(code) => return code,
    };

    let preview = match Previewed(assembly, &request.edit, &text, channels.notes)
    {
        Ok(preview) => preview,
        Err(code) => return code,
    };
    let _ = writeln!(channels.output, "{}", preview.Describe());

    return Committed(assembly, &Staged { preview, text }, request, channels);
}

/// An edit the store has already checked, and the bytes it was checked against.
///
/// The two travel together because the preview is what proves the edit admissible and the
/// text is what lands on disk, and writing one without the other is the half-done commit
/// this surface exists to prevent.
pub(super) struct Staged
{
    /// The edit as the store checked it.
    preview: EditPreview,
    /// The bytes that were staged.
    text: String,
}

/// The edit written where the author expects it, and the round trip closed behind it.
pub(super) fn Committed(
    assembly: &mut Assembly,
    staged: &Staged,
    request: &CommitRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let renamed = staged.preview.Rename().map(|(before, _)| return before.to_owned());
    let report = match assembly.store.Commit_Edit(&staged.preview)
    {
        Ok(report) => report,
        Err(error) => return Report_Edit_Error(assembly, &error, channels.notes),
    };

    let destination = request.into.join(&report.path);
    let vacated = renamed.map(|path| return request.into.join(path));
    if let Some(code) = Written(&destination, &staged.text, vacated.as_deref(), channels)
    {
        return code;
    }

    Report_Commit(&report, channels.output);

    return Reproduced(assembly, &request.edit.id, &staged.text, channels);
}

/// What the commit changed, counted.
pub(super) fn Report_Commit(report: &CommitReport, output: &mut dyn std::io::Write)
{
    let _ = writeln!(
        output,
        "committed {} to {}: {} block(s), {} removed, {} relation(s) added, {} removed",
        report.node_id,
        report.path,
        report.blocks,
        report.blocks_removed,
        report.relations_added,
        report.relations_removed
    );
}

/// Writes the committed record where the author expects it, and vacates the path a rename
/// left.
///
/// Deleting the old file is part of the rename rather than left to the author: two files
/// declaring one identifier is what `Test_Every_Canonical_Record_On_Disk_Should_Be_Governing`
/// would report as a phantom record, and a rename that needs a follow-up step is a rename
/// somebody will half-do.
pub(super) fn Written(
    destination: &Path,
    staged: &str,
    vacated: Option<&Path>,
    channels: &mut Channels<'_>,
) -> Option<ExitCode>
{
    if let Some(code) = Placed(destination, staged, channels.notes)
    {
        return Some(code);
    }

    if let Some(old) = vacated
    {
        Vacated(destination, old, channels);
    }

    return None;
}

/// The record's own bytes, under a directory that is made if it is not there.
pub(super) fn Placed(destination: &Path, staged: &str, notes: &mut dyn std::io::Write) -> Option<ExitCode>
{
    if let Some(parent) = destination.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        let _ = writeln!(notes, "cannot create {}: {error}", parent.display());

        return Some(ExitCode::Unwritable);
    }

    if let Err(error) = std::fs::write(destination, staged)
    {
        let _ = writeln!(notes, "cannot write {}: {error}", destination.display());

        return Some(ExitCode::Unwritable);
    }

    return None;
}

/// The path a rename left behind, removed.
///
/// A failure here is reported and not fatal: the new file is already written, so the run
/// succeeded at the edit and failed at the tidying, and saying so is more use than an exit
/// code that suggests nothing landed.
pub(super) fn Vacated(destination: &Path, old: &Path, channels: &mut Channels<'_>)
{
    match std::fs::remove_file(old)
    {
        Ok(()) => drop(writeln!(channels.output, "vacated {}", old.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => drop(writeln!(
            channels.notes,
            "{} was written and {} could not be removed ({error}), so two files now declare \
             this record",
            destination.display(),
            old.display()
        )),
    }
}

/// The round trip, closed on the way out: the store is asked to render what was just
/// committed, and the answer is compared with it.
pub(super) fn Reproduced(
    assembly: &Assembly,
    id: &str,
    staged: &str,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let _ = writeln!(channels.notes, "{EPHEMERAL}");

    return match assembly.store.Record_Markdown(id, None)
    {
        Ok(projection) if projection.markdown == staged =>
        {
            let _ = writeln!(
                channels.output,
                "the store renders it back as the same bytes ({})",
                projection.projected_hash
            );

            ExitCode::Ok
        }
        Ok(projection) =>
        {
            let _ = writeln!(
                channels.notes,
                "the commit succeeded and the store renders {} rather than what was committed, \
                 so the round trip does not close here",
                projection.projected_hash
            );

            ExitCode::Stale
        }
        Err(error) => Report_Edit_Error(assembly, &error, channels.notes),
    };
}

pub(super) fn Staged_Text(from: &Path, notes: &mut dyn std::io::Write) -> Result<String, ExitCode>
{
    return std::fs::read_to_string(from).map_err(|error| {
        let _ = writeln!(notes, "cannot read {}: {error}", from.display());

        return ExitCode::Usage;
    });
}

pub(super) fn Previewed(
    assembly: &Assembly,
    request: &EditRequest,
    staged: &str,
    notes: &mut dyn std::io::Write,
) -> Result<EditPreview, ExitCode>
{
    let rename = request.rename.as_deref();

    return assembly
        .store
        .Claim_For_Edit(&request.id, None)
        .and_then(|claimed| return claimed.Stage(staged, rename))
        .and_then(|edit| return edit.Preview(&assembly.store))
        .map_err(|error| return Report_Edit_Error(assembly, &error, notes));
}

/// The code an authoring refusal reports.
///
/// Everything the author can fix by editing their text is [`ExitCode::Refused`]; an
/// identifier the store does not hold goes through [`Absent_Or`], because over a store the
/// corpus never reached the honest answer is that something was missing.
pub(super) fn Report_Edit_Error(
    assembly: &Assembly,
    error: &EditError,
    notes: &mut dyn std::io::Write,
) -> ExitCode
{
    let _ = writeln!(notes, "{error}");

    return match error
    {
        EditError::NoSuchRecord { .. } | EditError::NoContent { .. } =>
        {
            Absent_Or(assembly, ExitCode::NotFound, notes)
        }
        EditError::Ambiguous { .. } => ExitCode::NotFound,
        EditError::Store(_) => ExitCode::StoreError,
        EditError::NotAuthored { .. }
        | EditError::Unreadable { .. }
        | EditError::IdentityChanged { .. }
        | EditError::NotCanonical { .. }
        | EditError::PathTaken { .. } => ExitCode::Refused,
    };
}
