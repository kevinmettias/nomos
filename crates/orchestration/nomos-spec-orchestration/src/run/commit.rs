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
//! Removing the path a rename left behind goes through the port too, now that it has a
//! fourth operation for it: [`FileSystem::Remove_File`]. Defaulted rather than required, so
//! adding it did not force every existing implementor -- several of them fakes built for one
//! narrow test elsewhere in this workspace -- to grow a removal they have no reason to
//! support. [`StdFileSystem`]'s own override is the real deletion this step performs; nothing
//! here calls `std::fs::remove_file` directly anymore.
//!
//! [`StdFileSystem`]: nomos_platform_std::StdFileSystem

use nomos_platform::FileSystem;
use nomos_spec_store::EditPreview;
use std::path::Path;

use crate::corpus::Assembly;
use crate::spec_outcome::{CommitAnswer, CommitRefusal, Reproduction};
use crate::request::CommitRequest;

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
// `EditPreview` at all -- the same trade `spec_outcome::record_answer`'s own `Record` already made and
// documented for `RecordRefusal::NotFound`'s unboxed `NodeSummary`.
#[allow(clippy::result_large_err)]
pub fn Commit_Staged_Edit<Filesystem: FileSystem>(
    assembly: &mut Assembly,
    request: &CommitRequest,
    filesystem: &Filesystem,
) -> Result<CommitAnswer, CommitRefusal>
{
    use crate::run::Preview_Staged_Edit;

    let preview = Preview_Staged_Edit(assembly, &request.edit, filesystem)?;
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

    let vacated = renamed.map(|old| return Vacate_Renamed_Path(&destination, &request.into.join(old), filesystem));
    let reproduction = Reproduction_Of(assembly, &preview);

    return Ok(CommitAnswer { preview, report, destination, vacated, reproduction });
}

/// The path a rename left behind, removed through the port.
///
/// A failure here is carried and not fatal: the new file is already written, so the commit
/// succeeded at the edit and only the tidying failed.
fn Vacate_Renamed_Path<Filesystem: FileSystem>(destination: &Path, old: &Path, filesystem: &Filesystem) -> crate::spec_outcome::Vacated
{
    use nomos_platform::FileSystemError;

    use crate::spec_outcome::{VacateOutcome, Vacated};

    let outcome = match filesystem.Remove_File(old)
    {
        Ok(()) => VacateOutcome::Removed,
        Err(FileSystemError::NotFound { .. }) => VacateOutcome::AlreadyGone,
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
fn Reproduction_Of(assembly: &Assembly, preview: &EditPreview) -> Result<Reproduction, nomos_spec_store::EditError>
{
    let projection = assembly.store.Record_Markdown(preview.Node_Id(), None)?;

    if projection.markdown == preview.Markdown()
    {
        return Ok(Reproduction::Matched { hash: projection.projected_hash });
    }

    return Ok(Reproduction::Mismatched { hash: projection.projected_hash });
}

#[cfg(test)]
mod tests
{
    use super::{CommitRequest, Commit_Staged_Edit};
    use crate::corpus::{Assemble_Corpus, Assembly, CorpusRequest};
    use crate::request::EditRequest;
    use nomos_platform_std::StdFileSystem;
    use std::path::PathBuf;

    #[test]
    fn Test_Commit_Staged_Edit_Should_Write_The_Record_And_Close_The_Round_Trip()
    {
        let mut assembly = Assembled();
        let into = Scratch("colocated");
        let markdown = crate::run::Rendered_Markdown(&assembly, &crate::request::RecordRequest { id: "D-132".to_owned(), revision: None })
            .expect("D-132 is embedded")
            .markdown;
        let edited = markdown.replace("## Decision", "## The decision");
        let staged = into.join("staged.md");
        std::fs::write(&staged, &edited).expect("writes the staged edit");

        let answer = Commit_Staged_Edit(&mut assembly, &CommitRequest { edit: EditRequest { id: "D-132".to_owned(), from: staged, rename: None }, into }, &StdFileSystem)
            .expect("a canonical heading rename commits cleanly");

        assert_eq!(answer.report.node_id, "D-132");
        let written = std::fs::read_to_string(&answer.destination).expect("the record was written");
        assert_eq!(written, edited, "the bytes on disk must be exactly what was staged");
        assert!(matches!(answer.reproduction, Ok(super::Reproduction::Matched { .. })), "the round trip must close: {:?}", answer.reproduction);
    }

    fn Assembled() -> Assembly
    {
        let request = CorpusRequest { variable: "A_COMMIT_TEST_CORPUS_VARIABLE".to_owned(), root: None, revision: "v14.36".to_owned() };
        return Assemble_Corpus(&request).expect("assembles from the embedded records alone");
    }

    fn Scratch(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(format!("nomos-spec-orchestration-commit-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch build root");
        return root;
    }
}
