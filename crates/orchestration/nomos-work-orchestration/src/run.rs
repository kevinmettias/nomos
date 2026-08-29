//! Running one [`WorkCommand`] against a ledger, generic over the platform it was built
//! with.

use std::path::Path;

use nomos_ledger::{
    ExclusionLedger, FileLedger, Finish, Finishing, ItemId, LedgerDocument, LedgerError,
    LedgerItem, ReleaseOutcome, Territory, Validate,
};
use nomos_platform::{Clock, CrossProcessLock, FileSystem, ProcessLauncher};

use crate::command::WorkCommand;
use crate::outcome::{BoardView, ShowView, WorkOutcome};
use crate::{ClaimRequest, EndingRequest};

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
///
/// The body is one `match` on `command`. `List`, `Show`, `Validate` and `Audit` need
/// nothing but `ledger`, so each wraps its helper's result in the [`WorkOutcome`] variant of
/// the same name right there. The other seven need `launcher`, `published`, or more than one
/// field off `command`, so each hands off to a per-command function below that owns both the
/// single ledger call and the [`WorkOutcome`] wrap -- naming what that arm already was,
/// rather than leaving `Run` itself carry every arm's own ledger call inline.
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
        WorkCommand::Add { item, amending } => Add_Outcome(ledger, item, amending, published),
        WorkCommand::Finish { item, holder } => Finish_Outcome(ledger, launcher, item, holder),
        WorkCommand::Claim(request) => Claim_Outcome(ledger, request),
        WorkCommand::Renew(request) => Renew_Outcome(ledger, request),
        WorkCommand::TakeOver(request) => TakeOver_Outcome(ledger, request),
        WorkCommand::Abandon(request) => Abandon_Outcome(ledger, request),
        WorkCommand::Decline(request) => Decline_Outcome(ledger, request),
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

/// The outcome of adding `item` to the board under `amending`, for [`WorkCommand::Add`].
fn Add_Outcome<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &mut FileLedger<F, C, L>,
    item: &LedgerItem,
    amending: &Territory,
    published: impl FnOnce() -> Territory,
) -> WorkOutcome
{
    let added = ledger.Add(item, "nomos work add", &published(), amending);

    return WorkOutcome::Add(added);
}

/// The outcome of running `item`'s verification predicate as `holder` claims it, for
/// [`WorkCommand::Finish`].
fn Finish_Outcome<F: FileSystem, C: Clock, L: CrossProcessLock, P: ProcessLauncher>(
    ledger: &mut FileLedger<F, C, L>,
    launcher: &P,
    item: &ItemId,
    holder: &str,
) -> WorkOutcome
{
    let finishing = Finishing { item, holder };
    let finished = Finish(ledger, launcher, &finishing, None);

    return WorkOutcome::Finish(finished);
}

/// The outcome of granting `request`, for [`WorkCommand::Claim`].
fn Claim_Outcome<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &mut FileLedger<F, C, L>,
    request: &ClaimRequest,
) -> WorkOutcome
{
    let claimed = ledger.Claim(&request.item, &request.holder, request.lease);

    return WorkOutcome::Claim(claimed);
}

/// The outcome of extending `request`'s lease, for [`WorkCommand::Renew`].
fn Renew_Outcome<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &mut FileLedger<F, C, L>,
    request: &ClaimRequest,
) -> WorkOutcome
{
    let renewed = ledger.Renew(&request.item, &request.holder, request.lease);

    return WorkOutcome::Renew(renewed);
}

/// The outcome of taking over `request`'s lapsed claim, for [`WorkCommand::TakeOver`].
fn TakeOver_Outcome<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &mut FileLedger<F, C, L>,
    request: &ClaimRequest,
) -> WorkOutcome
{
    let taken_over = ledger.Take_Over(&request.item, &request.holder, request.lease);

    return WorkOutcome::TakeOver(taken_over);
}

/// The outcome of giving up `request`'s claim without finishing it, for
/// [`WorkCommand::Abandon`].
fn Abandon_Outcome<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &mut FileLedger<F, C, L>,
    request: &EndingRequest,
) -> WorkOutcome
{
    let abandoned = ReleaseOutcome::Abandoned {
        reason: request.reason.clone(),
    };
    let released = ledger.Release(&request.item, &request.holder, abandoned);

    return WorkOutcome::Abandon(released);
}

/// The outcome of ending `request`'s item as not being work, for [`WorkCommand::Decline`].
fn Decline_Outcome<F: FileSystem, C: Clock, L: CrossProcessLock>(
    ledger: &mut FileLedger<F, C, L>,
    request: &EndingRequest,
) -> WorkOutcome
{
    let declined = ledger.Decline(&request.item, &request.holder, &request.reason);

    return WorkOutcome::Decline(declined);
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
