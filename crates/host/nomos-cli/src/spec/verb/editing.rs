//! Rendering `nomos spec preview`'s and `nomos spec commit`'s answers, or the refusal saying
//! why one has none.
//!
//! Staging the edit, checking it against the store and -- for a commit -- applying the
//! transaction and writing its bytes where the record belongs is
//! `nomos-spec-orchestration::{Preview, Commit}`'s job now, through
//! `nomos-platform-std::StdFileSystem` -- the same composition-root choice `nomos-cli::work`
//! already makes for the ledger. This module keeps only the writing of *text about* what
//! happened and the `ExitCode` a rendering layer is responsible for.

use crate::spec::{Assembly, EditRequest, Channels, ExitCode, EPHEMERAL, CommitRequest, EditPreview, CommitReport, Path, EditError, Absent_Or};
use nomos_platform::FileSystemError;
use nomos_platform_std::StdFileSystem;
use nomos_spec_orchestration::{
    CommitAnswer, CommitRefusal, CommitRefusalError, CommitRefusalKind, PreviewRefusal, Reproduction, VacateOutcome,
    Vacated,
};

/// The preview, printed, changing nothing.
pub(in crate::spec) fn Preview(assembly: &Assembly, request: &EditRequest, channels: &mut Channels<'_>) -> ExitCode
{
    return match nomos_spec_orchestration::Preview(assembly, request, &StdFileSystem)
    {
        Ok(preview) => Described(&preview, channels),
        Err(PreviewRefusal::Unreadable { path, error }) => Unreadable(&path, &error, channels.notes),
        Err(PreviewRefusal::Edit(error)) => Report_Edit_Error(assembly, &error, channels.notes),
    };
}

/// What committing this edit would change, printed, with nothing yet written.
fn Described(preview: &EditPreview, channels: &mut Channels<'_>) -> ExitCode
{
    let _ = writeln!(channels.output, "{}", preview.Describe());
    let _ = writeln!(channels.notes, "nothing was written. {EPHEMERAL}");

    return ExitCode::Ok;
}

/// The preview and then the commit, in that order, because the other order is not available.
pub(in crate::spec) fn Commit(
    assembly: &mut Assembly,
    request: &CommitRequest,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    return match nomos_spec_orchestration::Commit(assembly, request, &StdFileSystem)
    {
        Ok(answer) => Reported(assembly, &answer, channels),
        Err(CommitRefusal { kind: CommitRefusalKind::Unreadable { path }, error }) =>
        {
            Report_Unreadable_Refusal(&path, error, channels.notes)
        }
        Err(CommitRefusal { kind: CommitRefusalKind::Edit, error }) =>
        {
            Report_Edit_Refusal(assembly, error, channels.notes)
        }
        Err(CommitRefusal { kind: CommitRefusalKind::Refused { preview }, error }) =>
        {
            Report_Refused_Refusal(assembly, &preview, error, channels)
        }
        Err(CommitRefusal { kind: CommitRefusalKind::Unwritable { preview, report: _, path }, error }) =>
        {
            Report_Unwritable_Refusal(&preview, &path, error, channels)
        }
    };
}

/// What committing changed, once the store accepted the transaction and its bytes landed.
fn Reported(assembly: &Assembly, answer: &CommitAnswer, channels: &mut Channels<'_>) -> ExitCode
{
    let _ = writeln!(channels.output, "{}", answer.preview.Describe());
    Report_Commit(&answer.report, channels.output);
    if let Some(vacated) = &answer.vacated
    {
        Report_Vacated(vacated, channels);
    }
    let _ = writeln!(channels.notes, "{EPHEMERAL}");

    return match &answer.reproduction
    {
        Ok(Reproduction::Matched { hash }) =>
        {
            let _ = writeln!(channels.output, "the store renders it back as the same bytes ({hash})");

            ExitCode::Ok
        }
        Ok(Reproduction::Mismatched { hash }) =>
        {
            let _ = writeln!(
                channels.notes,
                "the commit succeeded and the store renders {hash} rather than what was \
                 committed, so the round trip does not close here"
            );

            ExitCode::Stale
        }
        Err(error) => Report_Edit_Error(assembly, error, channels.notes),
    };
}

/// What the commit changed, counted.
fn Report_Commit(report: &CommitReport, output: &mut dyn std::io::Write)
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

/// The path a rename vacated, reported as whatever became of removing it.
///
/// A failure here is reported and not fatal: the new file is already written, so the run
/// succeeded at the edit and failed at the tidying, and saying so is more use than an exit
/// code that suggests nothing landed.
fn Report_Vacated(vacated: &Vacated, channels: &mut Channels<'_>)
{
    match &vacated.outcome
    {
        VacateOutcome::Removed =>
        {
            let _ = writeln!(channels.output, "vacated {}", vacated.path.display());
        }
        VacateOutcome::AlreadyGone => (),
        VacateOutcome::Failed(message) =>
        {
            let _ = writeln!(channels.notes, "{message}");
        }
    }
}

/// `CommitRefusalKind::Unreadable`'s own error is always [`CommitRefusalError::FileSystem`]
/// -- see the constructor's own guarantee named in the `unreachable!` below.
fn Report_Unreadable_Refusal(path: &Path, error: CommitRefusalError, notes: &mut dyn std::io::Write) -> ExitCode
{
    let CommitRefusalError::FileSystem(error) = error else
    {
        // rust-panic: allow: CommitRefusal's constructors are the only way to build one, and
        // CommitRefusal::Unreadable always pairs CommitRefusalKind::Unreadable with a
        // CommitRefusalError::FileSystem; no other constructor produces this kind.
        unreachable!("CommitRefusalKind::Unreadable always carries a filesystem CommitRefusalError")
    };

    return Unreadable(path, &error, notes);
}

/// `CommitRefusalKind::Edit`'s own error is always [`CommitRefusalError::Edit`].
fn Report_Edit_Refusal(assembly: &Assembly, error: CommitRefusalError, notes: &mut dyn std::io::Write) -> ExitCode
{
    let CommitRefusalError::Edit(error) = error else
    {
        // rust-panic: allow: CommitRefusal's constructors are the only way to build one, and
        // CommitRefusal::Edit always pairs CommitRefusalKind::Edit with a
        // CommitRefusalError::Edit; no other constructor produces this kind.
        unreachable!("CommitRefusalKind::Edit always carries an edit CommitRefusalError")
    };

    return Report_Edit_Error(assembly, &error, notes);
}

/// `CommitRefusalKind::Refused`'s own error is always [`CommitRefusalError::Edit`].
fn Report_Refused_Refusal(
    assembly: &Assembly,
    preview: &EditPreview,
    error: CommitRefusalError,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let CommitRefusalError::Edit(error) = error else
    {
        // rust-panic: allow: CommitRefusal's constructors are the only way to build one, and
        // CommitRefusal::Refused always pairs CommitRefusalKind::Refused with a
        // CommitRefusalError::Edit; no other constructor produces this kind.
        unreachable!("CommitRefusalKind::Refused always carries an edit CommitRefusalError")
    };

    return Refused(assembly, preview, &error, channels);
}

/// An edit that previewed cleanly, printed, ahead of the store's own refusal to commit it.
///
/// The preview is shown regardless: an author who has already seen what their edit would
/// change should not lose that view because the store found a reason, after the fact, not to
/// apply it.
fn Refused(assembly: &Assembly, preview: &EditPreview, error: &EditError, channels: &mut Channels<'_>) -> ExitCode
{
    let _ = writeln!(channels.output, "{}", preview.Describe());

    return Report_Edit_Error(assembly, error, channels.notes);
}

/// `CommitRefusalKind::Unwritable`'s own error is always [`CommitRefusalError::FileSystem`].
fn Report_Unwritable_Refusal(
    preview: &EditPreview,
    path: &Path,
    error: CommitRefusalError,
    channels: &mut Channels<'_>,
) -> ExitCode
{
    let CommitRefusalError::FileSystem(error) = error else
    {
        // rust-panic: allow: CommitRefusal's constructors are the only way to build one, and
        // CommitRefusal::Unwritable always pairs CommitRefusalKind::Unwritable with a
        // CommitRefusalError::FileSystem; no other constructor produces this kind.
        unreachable!("CommitRefusalKind::Unwritable always carries a filesystem CommitRefusalError")
    };
    let _ = writeln!(channels.output, "{}", preview.Describe());

    return Unwritable(path, &error, channels.notes);
}

/// The store accepted the transaction and its bytes could not be written where the record
/// belongs.
fn Unwritable(path: &Path, error: &FileSystemError, notes: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "cannot write {}: {error}", path.display());

    return ExitCode::Unwritable;
}

/// The code an authoring refusal reports.
///
/// Everything the author can fix by editing their text is [`ExitCode::Refused`]; an
/// identifier the store does not hold goes through [`Absent_Or`], because over a store the
/// corpus never reached the honest answer is that something was missing.
pub(in crate::spec) fn Report_Edit_Error(
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

/// `--from` named a file this build could not read.
fn Unreadable(path: &Path, error: &FileSystemError, notes: &mut dyn std::io::Write) -> ExitCode
{
    let _ = writeln!(notes, "cannot read {}: {error}", path.display());

    return ExitCode::Usage;
}
