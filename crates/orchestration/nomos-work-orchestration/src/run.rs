//! Running one [`WorkCommand`] against a ledger, generic over the platform it was built
//! with.

use std::path::Path;

use nomos_ledger::{
    ExclusionLedger, FileLedger, Finish, Finishing, LedgerDocument, LedgerError, ReleaseOutcome,
    Territory, Validate,
};
use nomos_platform::{Clock, CrossProcessLock, FileSystem, ProcessLauncher};

use crate::command::WorkCommand;
use crate::outcome::{BoardView, ShowView, WorkOutcome};

/// Runs one command against `ledger` and hands back what happened, choosing nothing about
/// the platform and rendering nothing about the answer.
///
/// Generic over the traits [`nomos_platform`] declares and never over a concrete
/// implementation of them: a composition root builds `ledger` and `launcher` from whatever
/// it has — `nomos-cli` from `nomos-platform-std` today — and this function runs the same
/// way regardless. That is the seam `OD-HOST-001` asked for: a second composition root can
/// depend on this crate, build its own `F`, `C`, `L` and `P`, and call [`Run`] without also
/// taking on how `nomos-cli` chooses those four or how it prints an answer.
///
/// `published` is asked for lazily and only reached by [`WorkCommand::Add`]. The territory
/// this repository's own records already occupy is not answerable through [`FileSystem`] —
/// that port is read, atomically-replace and exists, not a directory walk — so a
/// composition root computes it however its own tree is reached and handed over as a
/// value, the same division `nomos-ledger::FileLedger::Add`'s own documentation already
/// draws around this exact question.
pub fn Run<F, C, L, P>(
    command: &WorkCommand,
    ledger: &mut FileLedger<F, C, L>,
    launcher: &P,
    published: impl FnOnce() -> Territory,
) -> WorkOutcome
where
    F: FileSystem,
    C: Clock,
    L: CrossProcessLock,
    P: ProcessLauncher,
{
    return match command
    {
        WorkCommand::List { .. } => WorkOutcome::List(Board_View(ledger)),
        WorkCommand::Show { .. } => WorkOutcome::Show(Show_View(ledger)),
        WorkCommand::Add { item, amending } =>
        {
            WorkOutcome::Add(ledger.Add(item, "nomos work add", &published(), amending))
        }
        WorkCommand::Finish { item, holder } =>
        {
            let finishing = Finishing { item, holder };
            WorkOutcome::Finish(Finish(ledger, launcher, &finishing, None))
        }
        WorkCommand::Claim(request) =>
        {
            WorkOutcome::Claim(ledger.Claim(&request.item, &request.holder, request.lease))
        }
        WorkCommand::Renew(request) =>
        {
            WorkOutcome::Renew(ledger.Renew(&request.item, &request.holder, request.lease))
        }
        WorkCommand::TakeOver(request) => WorkOutcome::TakeOver(ledger.Take_Over(
            &request.item,
            &request.holder,
            request.lease,
        )),
        WorkCommand::Abandon(request) =>
        {
            let abandoned = ReleaseOutcome::Abandoned {
                reason: request.reason.clone(),
            };
            WorkOutcome::Abandon(ledger.Release(&request.item, &request.holder, abandoned))
        }
        WorkCommand::Decline(request) => WorkOutcome::Decline(ledger.Decline(
            &request.item,
            &request.holder,
            &request.reason,
        )),
        WorkCommand::Validate => WorkOutcome::Validate(Validated(ledger)),
        WorkCommand::Audit => WorkOutcome::Audit(Board_View(ledger)),
    };
}

/// The board and the moment it was read, for `list` and `audit` alike.
fn Board_View<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &FileLedger<F, C, L>,
) -> Result<BoardView, LedgerError>
{
    let document = ledger.Load()?;
    let now = ledger.Now();

    return Ok(BoardView { document, now });
}

/// The board, the moment, and this tree's revision, for `show`.
fn Show_View<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &FileLedger<F, C, L>,
) -> Result<ShowView, LedgerError>
{
    let document = ledger.Load()?;
    let now = ledger.Now();
    let current_revision = Current_Revision(ledger);

    return Ok(ShowView {
        document,
        now,
        current_revision,
    });
}

/// The board, once it is known to satisfy its own invariants.
fn Validated<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &FileLedger<F, C, L>,
) -> Result<LedgerDocument, LedgerError>
{
    let document = ledger.Load()?;
    let violations = Validate(&document, ledger.Now());
    if !violations.is_empty()
    {
        return Err(LedgerError::Invalid { violations });
    }

    return Ok(document);
}

/// This tree's revision right now, read the same way [`nomos_ledger::Finish`] reads it when
/// it stamps a [`nomos_ledger::VerificationRecord`] — `.git/HEAD`, following one loose ref.
///
/// A second reading rather than a shared one: the resolution `nomos_ledger::Finish` uses to
/// stamp a record is private to that crate's `finish` module. `docs/records/OD-LEDGER-027-
/// ...md` says so, for the reading this moved from.
///
/// `None` on any failure — no `.git` here, a packed ref this build does not chase, or any
/// other read error. `show`'s staleness line treats that as its own case rather than as
/// agreement with a recorded revision.
fn Current_Revision<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &FileLedger<F, C, L>,
) -> Option<String>
{
    let head = ledger.Read_File(Path::new(".git/HEAD")).ok()?;
    let head = head.trim();

    if let Some(ref_path) = head.strip_prefix("ref: ")
    {
        return ledger
            .Read_File(&Path::new(".git").join(ref_path))
            .ok()
            .map(|contents| return contents.trim().to_owned());
    }

    return Some(head.to_owned());
}
