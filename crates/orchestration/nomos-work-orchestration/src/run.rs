//! Running one [`WorkCommand`] against a ledger, generic over the platform it was built
//! with.

use nomos_ledger::{
    ExclusionLedger, FileLedger, Finish_Item, Finishing, ItemId, LedgerDocument, LedgerError,
    LedgerItem, ReleaseOutcome, Territory, Validate_Document,
};
use nomos_platform::{Clock, CrossProcessLock, FileSystem, ProcessLauncher};

use crate::board_view::{BoardView, ShowView, WorkOutcome};
use crate::{ClaimRequest, EndingRequest, WorkCommand};

/// Runs one command against `ledger` and hands back what happened, choosing nothing about
/// the platform and rendering nothing about the answer.
///
/// Generic over the traits [`nomos_platform`] declares and never over a concrete
/// implementation of them: a composition root builds `ledger` and `launcher` from whatever
/// it has — `nomos-cli` from `nomos-platform-std` today — and this function runs the same
/// way regardless. That is the seam `OD-HOST-001` asked for: a second composition root can
/// depend on this crate, build its own `Filesystem`, `ClockSource`, `Lock` and `Launcher`,
/// and call [`Run`] without also taking on how `nomos-cli` chooses those four or how it
/// prints an answer.
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
pub fn Run<Filesystem, ClockSource, Lock, Launcher>(
    command: &WorkCommand,
    ledger: &mut FileLedger<Filesystem, ClockSource, Lock>,
    launcher: &Launcher,
    published: impl FnOnce() -> Territory,
) -> WorkOutcome
where
    Filesystem: FileSystem,
    ClockSource: Clock,
    Lock: CrossProcessLock,
    Launcher: ProcessLauncher,
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
        WorkCommand::Validate => WorkOutcome::Validate(Validated_Board(ledger)),
        WorkCommand::Audit => WorkOutcome::Audit(Board_View(ledger)),
    };
}

/// The board and the moment it was read, for `list` and `audit` alike.
fn Board_View<Filesystem: FileSystem, ClockSource: Clock, Lock: CrossProcessLock>(
    ledger: &FileLedger<Filesystem, ClockSource, Lock>,
) -> Result<BoardView, LedgerError>
{
    let document = ledger.Load()?;
    let now = ledger.Now();

    return Ok(BoardView { document, now });
}

/// The board, the moment, and this tree's revision, for `show`.
fn Show_View<Filesystem: FileSystem, ClockSource: Clock, Lock: CrossProcessLock>(
    ledger: &FileLedger<Filesystem, ClockSource, Lock>,
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

/// This tree's revision right now, read the same way [`nomos_ledger::Finish_Item`] reads it when
/// it stamps a [`nomos_ledger::VerificationRecord`] — `.git/HEAD`, following one loose ref.
///
/// A second reading rather than a shared one: the resolution `nomos_ledger::Finish_Item` uses to
/// stamp a record is private to that crate's `finish` module. `docs/records/OD-LEDGER-027-
/// ...md` says so, for the reading this moved from.
///
/// `None` on any failure — no `.git` here, a packed ref this build does not chase, or any
/// other read error. `show`'s staleness line treats that as its own case rather than as
/// agreement with a recorded revision.
fn Current_Revision<Filesystem: FileSystem, ClockSource: Clock, Lock: CrossProcessLock>(
    ledger: &FileLedger<Filesystem, ClockSource, Lock>,
) -> Option<String>
{
    use std::path::Path;

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
fn Add_Outcome<Filesystem: FileSystem, ClockSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Filesystem, ClockSource, Lock>,
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
fn Finish_Outcome<
    Filesystem: FileSystem,
    ClockSource: Clock,
    Lock: CrossProcessLock,
    Launcher: ProcessLauncher,
>(
    ledger: &mut FileLedger<Filesystem, ClockSource, Lock>,
    launcher: &Launcher,
    item: &ItemId,
    holder: &str,
) -> WorkOutcome
{
    let finishing = Finishing { item, holder };
    let finished = Finish_Item(ledger, launcher, &finishing, None);

    return WorkOutcome::Finish(finished);
}

/// The outcome of granting `request`, for [`WorkCommand::Claim`].
fn Claim_Outcome<Filesystem: FileSystem, ClockSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Filesystem, ClockSource, Lock>,
    request: &ClaimRequest,
) -> WorkOutcome
{
    let claimed = ledger.Claim(&request.item, &request.holder, request.lease);

    return WorkOutcome::Claim(claimed);
}

/// The outcome of extending `request`'s lease, for [`WorkCommand::Renew`].
fn Renew_Outcome<Filesystem: FileSystem, ClockSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Filesystem, ClockSource, Lock>,
    request: &ClaimRequest,
) -> WorkOutcome
{
    let renewed = ledger.Renew(&request.item, &request.holder, request.lease);

    return WorkOutcome::Renew(renewed);
}

/// The outcome of taking over `request`'s lapsed claim, for [`WorkCommand::TakeOver`].
fn TakeOver_Outcome<Filesystem: FileSystem, ClockSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Filesystem, ClockSource, Lock>,
    request: &ClaimRequest,
) -> WorkOutcome
{
    let taken_over = ledger.Take_Over(&request.item, &request.holder, request.lease);

    return WorkOutcome::TakeOver(taken_over);
}

/// The outcome of giving up `request`'s claim without finishing it, for
/// [`WorkCommand::Abandon`].
fn Abandon_Outcome<Filesystem: FileSystem, ClockSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Filesystem, ClockSource, Lock>,
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
fn Decline_Outcome<Filesystem: FileSystem, ClockSource: Clock, Lock: CrossProcessLock>(
    ledger: &mut FileLedger<Filesystem, ClockSource, Lock>,
    request: &EndingRequest,
) -> WorkOutcome
{
    let declined = ledger.Decline(&request.item, &request.holder, &request.reason);

    return WorkOutcome::Decline(declined);
}

/// The board, once it is known to satisfy its own invariants.
fn Validated_Board<Filesystem: FileSystem, ClockSource: Clock, Lock: CrossProcessLock>(
    ledger: &FileLedger<Filesystem, ClockSource, Lock>,
) -> Result<LedgerDocument, LedgerError>
{
    let document = ledger.Load()?;
    let violations = Validate_Document(&document, ledger.Now());
    if !violations.is_empty()
    {
        return Err(LedgerError::Invalid { violations });
    }

    return Ok(document);
}

#[cfg(test)]
mod tests
{
    use super::Run;
    use crate::WorkCommand;
    use nomos_ledger::{FileLedger, Territory};
    use nomos_platform_std::{FileLock, StdFileSystem, SystemClock};

    /// A ledger under a directory unique to this process and this test -- the same colocated
    /// shape [`crate::tests`]'s own `Scratch_Ledger` builds, re-homed here so this check's own
    /// companion rule (the test must sit beside `Run`'s own file) can find it.
    fn Scratch_Ledger() -> FileLedger<StdFileSystem, SystemClock, FileLock>
    {
        let root = std::env::temp_dir().join(format!("nomos-work-orchestration-run-colocated-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");

        return FileLedger::At(root.join("ledger.json"), StdFileSystem, SystemClock, FileLock::At(root.join("ledger.lock")));
    }

    /// No process is ever actually launched by `list`, so any launcher would do; one that
    /// panics if called also proves it.
    struct Unreached;

    impl nomos_platform::ProcessLauncher for Unreached
    {
        fn Run(&self, _command: &nomos_platform::Command) -> Result<nomos_platform::ProcessOutput, String>
        {
            panic!("no command dispatched by this test should run a process");
        }
    }

    #[test]
    fn Test_Run_Should_Dispatch_List_To_An_Empty_Board()
    {
        let mut ledger = Scratch_Ledger();

        let outcome = Run(&WorkCommand::List { state: None }, &mut ledger, &Unreached, Territory::Empty);

        let super::WorkOutcome::List(Ok(view)) = outcome
        else
        {
            panic!("an unwritten ledger loads as an empty, valid board");
        };
        assert!(view.document.items.is_empty());
    }
}
