//! Resolving `nomos spec commit`'s request: previewing the edit (see [`crate::run::preview`]),
//! committing it to the store, and writing what the store accepted where the record's own
//! path says.
//!
//! # Why the destination write is generic over [`FileSystem`], and vacating a rename is not
//!
//! Writing the committed bytes to `into.join(&report.path)` is exactly
//! [`crate::run::render`]'s own shape: a single-file write at a path this crate computes from
//! a caller-known root and a store-produced relative path, going through
//! [`FileSystem::Replace_Atomically`] for the same reason -- a truncating write has a window
//! in which the file is empty or half-written, and this replaces one that had it.
//!
//! Removing the path a rename left behind does not go through the port at all, because there
//! is nothing in it to go through: [`FileSystem`] declares three operations -- read,
//! atomically replace, and check existence -- and deletion, by its own documentation, is
//! deliberately not a fourth. That is not a gap this crate's territory can close (extending
//! `nomos-platform` is out of scope here), so vacating stays a direct `std::fs::remove_file`
//! call, the same way [`crate::corpus::Assemble`]'s own directory walk stays outside the port
//! for an operation it does not cover either.

use nomos_platform::FileSystem;
use nomos_spec_store::EditPreview;
use std::path::Path;

use crate::corpus::Assembly;
use crate::outcome::{CommitAnswer, CommitRefusal, Reproduction, VacateOutcome, Vacated};
use crate::request::CommitRequest;
use crate::run::preview::Preview;

/// Previews `request.edit`, commits it to the store, and writes it where its own path says.
///
/// Moved from `nomos-cli::spec::verb::editing`'s own `Commit`, `Committed`, `Written`,
/// `Placed`, `Vacated` and `Reproduced`. The preview and then the commit, in that order,
/// because the other order is not available:
/// [`nomos_spec_store::SpecificationStore::Commit_Edit`] takes an [`EditPreview`], a value
/// nothing outside `nomos-spec-store` can construct.
///
/// # Errors
///
/// Returns [`CommitRefusal::Unreadable`] when `request.edit.from` could not be read,
/// [`CommitRefusal::Edit`] when staging or previewing the edit was refused before there was
/// anything to commit, [`CommitRefusal::Refused`] when the store refused a cleanly previewed
/// edit, and [`CommitRefusal::Unwritable`] when the store accepted the transaction and its
/// bytes could not be written where the record belongs.
// `CommitRefusal::Refused` and `::Unwritable` each carry the `EditPreview` a caller already
// has to be shown, for the reason `run::preview`'s own module documentation gives for reusing
// `EditPreview` at all -- the same trade `outcome::record`'s own `Record` already made and
// documented for `RecordRefusal::NotFound`'s unboxed `NodeSummary`.
#[allow(clippy::result_large_err)]
pub fn Commit<F: FileSystem>(
    assembly: &mut Assembly,
    request: &CommitRequest,
    filesystem: &F,
) -> Result<CommitAnswer, CommitRefusal>
{
    let preview = Preview(assembly, &request.edit, filesystem)?;
    let renamed = preview.Rename().map(|(before, _)| return before.to_owned());

    let report = match assembly.store.Commit_Edit(&preview)
    {
        Ok(report) => report,
        Err(error) => return Err(CommitRefusal::Refused(preview, error)),
    };

    let destination = request.into.join(&report.path);
    if let Err(error) = filesystem.Replace_Atomically(&destination, preview.Markdown())
    {
        return Err(CommitRefusal::Unwritable(preview, report, destination, error));
    }

    let vacated = renamed.map(|old| return Vacate(&destination, &request.into.join(old)));
    let reproduction = Reproduced(assembly, &preview);

    return Ok(CommitAnswer { preview, report, destination, vacated, reproduction });
}

/// The path a rename left behind, removed.
///
/// A failure here is carried and not fatal: the new file is already written, so the commit
/// succeeded at the edit and only the tidying failed.
fn Vacate(destination: &Path, old: &Path) -> Vacated
{
    let outcome = match std::fs::remove_file(old)
    {
        Ok(()) => VacateOutcome::Removed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => VacateOutcome::AlreadyGone,
        Err(error) => VacateOutcome::Failed(format!(
            "{} was written and {} could not be removed ({error}), so two files now declare \
             this record",
            destination.display(),
            old.display()
        )),
    };

    return Vacated { path: old.to_owned(), outcome };
}

/// The round trip, closed on the way out: the store is asked to render what was just
/// committed, and the answer is compared with it.
fn Reproduced(assembly: &Assembly, preview: &EditPreview) -> Result<Reproduction, nomos_spec_store::EditError>
{
    let projection = assembly.store.Record_Markdown(preview.Node_Id(), None)?;

    if projection.markdown == preview.Markdown()
    {
        return Ok(Reproduction::Matched { hash: projection.projected_hash });
    }

    return Ok(Reproduction::Mismatched { hash: projection.projected_hash });
}
