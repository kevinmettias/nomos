//! `nomos work` — the ledger, from a terminal.
//!
//! This module is the composition root and the renderer, and nothing else. It chooses the
//! platform — `nomos-composer-std`'s `FILE_SYSTEM`, `CLOCK`, `Lock_At` and `LAUNCHER` —
//! over the two files `nomos_ledger::Board_In` names, which are the ledger's own format and
//! never this module's to restate —
//! and it turns a [`nomos_work_orchestration::WorkOutcome`] into
//! text and an [`ExitCode`]. Running the verb itself, between those two steps, is
//! `nomos_work_orchestration::Run`'s job: generic over the platform traits rather than
//! these four concrete types, so a second adapter can call it with its own choices without
//! also taking on how this module renders an answer. `OD-HOST-001`.

use nomos_ledger::{AddRefusal, Board_In, FileLedger, ItemId, LedgerDocument, LedgerError, LedgerItem, Territory};
use nomos_platform::{Clock, FileSystem};
use nomos_composer_std::{CLOCK, FILE_SYSTEM, LAUNCHER, Lock_At};
use nomos_work_orchestration::{BoardView, ShowView, WorkOutcome};
use std::path::Path;

mod exit_code;
mod listing;
mod parse;
mod report;
#[cfg(test)]
mod tests;

pub use parse::Work_Command_From_String_Arguments;

pub(crate) use exit_code::ExitCode;
pub(crate) use nomos_work_orchestration::{ClaimRequest, EndingRequest, ListingScope, WorkCommand};

use listing::{
    Bounds, Listed_As, Listing_Label, Nothing_Listed, Print_Claim, Print_Contract, Print_History,
    Print_Listing,
};
use report::{
    Amendment_Note, Blocking_Refusal, Code_For_Refusal, Ended, Print_Blocked, Report_Claim,
    Report_Decline, Report_Error, Report_Finish, Report_Release, Report_Validation, Report_Widen,
};

/// Runs a command against the ledger at `directory`, writing to `output`.
///
/// Returns the exit code rather than exiting, so the whole surface is testable.
pub fn Run(
    command: &WorkCommand,
    directory: &Path,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let board = Board_In(directory);
    let mut ledger = FileLedger::At(board.document, FILE_SYSTEM, CLOCK, Lock_At(board.lock));

    let outcome = nomos_work_orchestration::Run(
        command,
        &mut ledger,
        &LAUNCHER,
        || Published_Records(directory, &FILE_SYSTEM),
    );

    return Render_Outcome(command, outcome, directory, output);
}

/// Every record this repository has already published, as repository-relative paths.
///
/// Read here rather than in the store, and that division is the point rather than a
/// convenience. `OD-LEDGER-021` put the *decision* behind the ledger's lock and left the
/// command layer its input and its reporting; enumerating a repository is input. A general
/// exclusion ledger that learned to walk a source tree would be answering a question about
/// this repository's conventions, and `nomos-ledger` already stretches as far as it should
/// by knowing what a record filename folds to.
///
/// `OD-PLATFORM-002` gave [`nomos_platform::FileSystem`] a directory-listing primitive,
/// [`FileSystem::Read_Directory`], so a generic dispatch function could now reach this
/// through the port; this stays a composition-root function regardless, for the reason the
/// paragraph above already gives — the input this hands across `nomos_work_orchestration::
/// Run`'s `published` parameter is a repository's own convention about where records live,
/// not a ledger-agnostic filesystem concern the orchestration crate should own.
///
/// An unreadable or absent directory yields nothing rather than refusing. That is the one
/// judgement here worth stating, because this repository's usual rule is the opposite: a
/// check that cannot find its subject must fail loudly. It does not apply, because this is
/// not the check — the open-item comparison still runs, and it is the half that races. A
/// tree with no `docs/records` is a ledger being used somewhere that has no records, and
/// refusing every `add` in it would be this repository's convention refusing everybody
/// else's.
fn Published_Records(directory: &Path, filesystem: &impl FileSystem) -> Territory
{
    // The ledger lives in `work/`, so the repository is its parent. A `work/` at the root of
    // nothing has no records, which the walk below reports as none.
    let Some(root) = directory.parent()
    else
    {
        return Territory::Empty();
    };

    let mut published = Record_Files(root, filesystem);
    // Sorted so that an item colliding with two records is refused against the same one
    // every run. A refusal that names a different file each time reads as two defects.
    published.sort();

    return Territory::Of_Files(published);
}

/// Every file directly under the repository's record directory, as a territory is spelled.
///
/// Repository-relative and forward-slashed, which is the spelling a territory is authored
/// in. `Normalize_Path` would accept either, and handing it the shape it documents keeps the
/// refusal's text readable by whoever has to act on it.
fn Record_Files(root: &Path, filesystem: &impl FileSystem) -> Vec<String>
{
    let Ok(entries) = filesystem.Read_Directory(&root.join(RECORD_DIRECTORY))
    else
    {
        return Vec::new();
    };

    let mut published = Vec::new();
    for entry in entries
    {
        if let Some(name) = entry.file_name().and_then(std::ffi::OsStr::to_str)
        {
            published.push(format!("{RECORD_DIRECTORY}/{name}"));
        }
    }

    return published;
}

/// Where this repository authors its decision records, relative to the repository root.
const RECORD_DIRECTORY: &str = "docs/records";

#[cfg(test)]
mod published_records_tests
{
    use super::{Published_Records, RECORD_DIRECTORY};
    use nomos_ledger::Territory;
    use nomos_composer_std::FILE_SYSTEM;
    use std::path::PathBuf;

    #[test]
    fn Test_Published_Records_Should_List_Every_File_Directly_Under_The_Record_Directory()
    {
        let root = Fresh_Root("nomos-cli-work-published-records");
        std::fs::create_dir_all(root.join(RECORD_DIRECTORY))
            .expect("the test root is a scratch path outside the repository, so the directory can be made here");
        std::fs::write(root.join(RECORD_DIRECTORY).join("OD-EXAMPLE-001.md"), "# example")
            .expect("the record directory was created on the line above, so this record file is writable");
        std::fs::write(root.join(RECORD_DIRECTORY).join("OD-EXAMPLE-002.md"), "# example")
            .expect("the record directory was created on the line above, so this record file is writable");

        let territory = Published_Records(&root.join("work"), &FILE_SYSTEM);

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(
            territory,
            Territory::Of_Files(vec![
                format!("{RECORD_DIRECTORY}/OD-EXAMPLE-001.md"),
                format!("{RECORD_DIRECTORY}/OD-EXAMPLE-002.md"),
            ])
        );
    }

    #[test]
    fn Test_Published_Records_Should_Be_Empty_For_A_Repository_With_No_Record_Directory()
    {
        let root = Fresh_Root("nomos-cli-work-published-records-absent");

        let territory = Published_Records(&root.join("work"), &FILE_SYSTEM);

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(territory, Territory::Empty());
    }

    fn Fresh_Root(name: &str) -> PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("the temporary root is creatable");
        return root;
    }
}

/// Turns what [`nomos_work_orchestration::Run`] produced into text and an [`ExitCode`].
///
/// One `match` over `(command, outcome)` rather than a second dispatch on `command` alone:
/// `outcome`'s variant is already the answer to which arm this is, and matching both
/// together is what lets the compiler check that every arm actually reads the outcome
/// shape the command it is paired with produces. The last arm is the one Rust cannot see is
/// empty -- `nomos_work_orchestration::Run` returns the one `WorkOutcome` variant naming the
/// `WorkCommand` variant it was given -- so it reports that pairing coming apart rather than
/// panicking on it.
fn Render_Outcome(command: &WorkCommand, outcome: WorkOutcome, directory: &Path, output: &mut impl std::io::Write) -> ExitCode
{
    return match (command, outcome)
    {
        (WorkCommand::List { state, scope }, WorkOutcome::List(result)) =>
        {
            Render_List(Bounds { state: state.as_deref(), scope: *scope }, result, output)
        }
        (WorkCommand::Show { item }, WorkOutcome::Show(result)) => Render_Show(item, result, output),
        (WorkCommand::Add { item, amending }, WorkOutcome::Add(result)) =>
        {
            Render_Add(item, amending, result, output)
        }
        (WorkCommand::Finish { item, .. }, WorkOutcome::Finish { finished, board }) =>
        {
            Report_Finish(&Ended { item, board: board.as_ref(), now: CLOCK.Now() }, finished, output)
        }
        (WorkCommand::Claim(_), WorkOutcome::Claim(result))
        | (WorkCommand::Renew(_), WorkOutcome::Renew(result))
        | (WorkCommand::TakeOver(_), WorkOutcome::TakeOver(result)) => Report_Claim(result, output),
        (WorkCommand::Abandon(_), WorkOutcome::Abandon(result)) => Report_Release(result, output),
        (WorkCommand::Widen { item, .. }, WorkOutcome::Widen(result)) =>
        {
            Report_Widen(item, result, output)
        }
        (WorkCommand::Decline(request), WorkOutcome::Decline { declined, board }) =>
        {
            let ended =
                Ended { item: &request.item, board: board.as_ref(), now: CLOCK.Now() };
            Report_Decline(&ended, declined, output)
        }
        (WorkCommand::Validate, WorkOutcome::Validate(result)) => Report_Validation(result, output),
        (WorkCommand::Audit, WorkOutcome::Audit(result)) => Render_Audit(result, directory, output),
        // `Run` above builds each `WorkOutcome` from the variant it dispatched on, so this arm
        // is that pairing coming apart rather than an input a caller could have got right.
        (_, _) => Report_Unmatched_Outcome(command, output),
    };
}

fn Render_List(
    bounds: Bounds<'_>,
    result: Result<BoardView, LedgerError>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let BoardView { document, now } = match result
    {
        Ok(view) => view,
        Err(error) => return Report_Error(&error, output),
    };

    Print_Board(&document, bounds, now, output);

    return ExitCode::Ok;
}

/// Every item `bounds` admits, or the "nothing" line when none was -- and, on the unfiltered
/// board only, how many rows the bound withheld and the one line naming which item to claim
/// next.
///
/// `document` is the whole board at every step, and that is a constraint rather than an
/// incidental argument. [`Listing_Label`] reads a dependency's terminal state to decide
/// whether the row above it reads `ready`, `waiting` or `stranded`, and [`Print_Next`] runs
/// `Eligible_Items` over every item to say which one to claim -- so the bound governs what is
/// *printed* and never what is *read*. `OD-LEDGER-023` is why: what to work on next is
/// computed from the board rather than read off it, and a computation needs the whole board
/// present. A view does not, which is the whole of what `OD-LEDGER-041` asked for here.
fn Print_Board(
    document: &LedgerDocument,
    bounds: Bounds<'_>,
    now: nomos_platform::Timestamp,
    output: &mut impl std::io::Write,
)
{
    let mut shown = 0_u32;
    for item in &document.items
    {
        let Some(label) = Listed_As(document, item, bounds, now)
        else
        {
            continue;
        };
        Print_Listing(item, label, output);
        shown = shown.saturating_add(1);
    }
    if shown == 0
    {
        Nothing_Listed(bounds, output);
    }
    // Only on the unfiltered board. `--state` asks for one bucket's rows, and a summary
    // naming an item outside that bucket would contradict the very filter the caller asked
    // for -- `--state lapsed` is not the place to also learn what is `Ready` elsewhere.
    if bounds.state.is_none()
    {
        Print_Withheld(document, bounds.scope, output);
        Print_Next(document, now, output);
    }
}

/// How many rows the default's bound left out, and the flag that prints them.
///
/// A bound nobody is told about is indistinguishable from a board that has lost its history,
/// and `OD-LEDGER-041` keeps that history precisely so it can still be read -- so the line
/// that hides it also says how to ask for it.
///
/// Counted off the document rather than off the loop above. Under [`ListingScope::Live`] with
/// no `--state` those are the same number, and this one cannot later be made wrong by a
/// filter's own exclusions being added into it.
///
/// Silent under [`ListingScope::Whole`], and silent on a board with nothing ended: neither
/// withheld anything, and a line reporting zero is a line every reader has to stop and check.
fn Print_Withheld(document: &LedgerDocument, scope: ListingScope, output: &mut impl std::io::Write)
{
    if scope != ListingScope::Live
    {
        return;
    }

    let withheld = document.items.iter().filter(|item| return item.state.Is_Finished()).count();
    if withheld == 0
    {
        return;
    }

    let _ = writeln!(
        output,
        "{withheld} ended items not shown; `nomos work list --all` prints the whole board"
    );
}

/// The one line that answers "which one": the first item [`nomos_ledger::Eligible_Items`]
/// computes from the whole board.
///
/// `OD-LEDGER-023` is why this is a line on `list` rather than a verb of its own —
/// `WorkCommand` and its parser are outside this item's territory, and a real computed
/// answer delivered through the one listing surface this item can reach beats a dedicated
/// verb this item cannot wire up.
fn Print_Next(document: &LedgerDocument, now: nomos_platform::Timestamp, output: &mut impl std::io::Write)
{
    let _ = match nomos_ledger::Eligible_Items(document, now).first()
    {
        Some(item) => writeln!(output, "next: {} {}", item.id, item.title),
        None => writeln!(output, "next: nothing is eligible"),
    };
}

/// Reports one item: what has happened to it, and then the terms it is held to.
///
/// `list` is one line per item and cannot carry prose. That is why an abandonment had
/// nowhere to be read even once the ledger began keeping one: a record no surface reports
/// is a record only somebody willing to read the JSON can find, which is most of the way
/// back to not keeping it.
///
/// [`Print_Contract`] is that same argument reaching the four fields it had never been
/// applied to. `AGENTS.md`'s loop sends a session here to read an item's `why`, `done_when`,
/// territory and predicate before it edits anything, and this verb used to print none of
/// them.
fn Render_Show(
    item: &ItemId,
    result: Result<ShowView, LedgerError>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let ShowView { document, now, current_revision } = match result
    {
        Ok(view) => view,
        Err(error) => return Report_Error(&error, output),
    };

    let Some(found) = document.items.iter().find(|candidate| return &candidate.id == item)
    else
    {
        let _ = writeln!(output, "no item named {item}");
        return ExitCode::Conflict;
    };
    let label = Listing_Label(&document, found, now);

    let _ = writeln!(output, "{} {}", found.id, found.title);
    let _ = writeln!(output, "state: {label}");
    let _ = writeln!(output, "kind: {:?}  origin: {:?}", found.kind, found.origin);
    Print_Claim(found, now, output);
    Print_History(found, current_revision.as_deref(), output);
    Print_Contract(found, output);

    return ExitCode::Ok;
}

fn Render_Add(
    item: &LedgerItem,
    amending: &Territory,
    result: Result<(), AddRefusal>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    return match result
    {
        Ok(()) =>
        {
            let _ = writeln!(
                output,
                "added {} reserving {} path(s){}",
                item.id,
                item.territory.paths.len(),
                Amendment_Note(amending)
            );
            ExitCode::Ok
        }
        Err(refusal) =>
        {
            let _ = writeln!(output, "{}", refusal.Describe());

            Code_For_Refusal(&refusal)
        }
    };
}

fn Render_Audit(result: Result<BoardView, LedgerError>, directory: &Path, output: &mut impl std::io::Write) -> ExitCode
{
    let BoardView { document, now } = match result
    {
        Ok(view) => view,
        Err(error) => return Report_Error(&error, output),
    };

    Print_Audit(&document, now, output);

    // The ledger lives in `work/`, so the repository is its parent. A `work/` at the root
    // of nothing has no tree to check a reservation against, and `None` reads as "report no
    // absent paths" rather than "fail to audit".
    Print_Absent_Paths(&document, directory.parent(), &FILE_SYSTEM, output);

    return ExitCode::Ok;
}

/// Every claimable item currently blocked, or the "nothing" line when none is.
fn Print_Audit(document: &LedgerDocument, now: nomos_platform::Timestamp, output: &mut impl std::io::Write)
{
    let mut reported = 0_u32;
    for item in &document.items
    {
        let Some(refusal) = Blocking_Refusal(document, item, now)
        else
        {
            continue;
        };
        Print_Blocked(item, &refusal, output);
        reported = reported.saturating_add(1);
    }
    if reported == 0
    {
        // Empty output used to mean either "nothing is blocked" or "the report cannot
        // express what is blocking this", and only one of those is good news.
        let _ = writeln!(output, "nothing claimable is blocked");
    }
}

/// Every reserved path that is not in the tree, on an item that could still be worked.
///
/// `root` and `filesystem` answer the question the blocked report above cannot: a reservation
/// names paths the tree moved under, and nothing in the document says whether those paths
/// still exist. They are threaded here rather than read from ambient state, the same
/// `OD-HOST-001` division `Published_Records` already respects.
fn Print_Absent_Paths(
    document: &LedgerDocument,
    root: Option<&Path>,
    filesystem: &impl FileSystem,
    output: &mut impl std::io::Write,
)
{
    // A reservation names paths the tree moved under. An item that is Done or Declined is
    // history, and a stale path in it is not debt; every other item could still be worked,
    // so a path its territory reserves that is no longer in the tree is reported here.
    let Some(root) = root
    else
    {
        return;
    };
    for item in &document.items
    {
        if item.state.Is_Finished()
        {
            continue;
        }
        for absent in item.territory.Absent_Paths(root, filesystem)
        {
            Print_Absent_Path(item, &absent, output);
        }
    }
}

/// One absent reserved path: the path that is not in the tree, and the item that reserves it.
/// Authored as *absent* rather than *decayed* — the line states the fact and does not guess
/// whether the item means to create the file or a peer moved it, because a report that guessed
/// would cry wolf on a board that already contains the first kind. Indented and path-first so
/// it reads as a second section rather than as one more blocked item.
fn Print_Absent_Path(item: &nomos_ledger::LedgerItem, absent: &str, output: &mut impl std::io::Write)
{
    let _ = writeln!(output, "  {absent} is not in the tree (reserved by {})", item.id);
}

/// A `WorkOutcome` whose variant does not belong to the `WorkCommand` standing beside it.
///
/// [`Render_Outcome`]'s catch-all arm, named so the arm reads as one thing and the reporting
/// it does has somewhere to live. Reported rather than panicked: the caller gets a message and
/// an exit code either way, and a panic would give it neither.
fn Report_Unmatched_Outcome(command: &WorkCommand, output: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(
        output,
        "the ledger answered {command:?} with an outcome of another verb's shape"
    );

    return ExitCode::StoreError;
}

/// This module's own composition-root test.
///
/// `Run` is not generic over the platform traits the way `nomos_work_orchestration::Run` is --
/// it always chooses `FILE_SYSTEM`, `CLOCK` and `Lock_At`, which is the whole point of
/// this file (`OD-HOST-001`). Proving that wiring holds together means running it against a
/// real directory on disk rather than a fake, because a fake is exactly the thing this
/// composition root does not accept.
#[cfg(test)]
mod run_test
{
    use super::*;

    #[test]
    fn Test_Run_Should_Validate_A_Real_Ledger_Through_The_Chosen_Platform()
    {
        let directory = Scratch_Directory("validate");
        let mut output = Vec::new();

        let code = Run(&WorkCommand::Validate, &directory, &mut output);

        let _ignored = std::fs::remove_dir_all(&directory);

        assert_eq!(code, ExitCode::Ok);
        assert!(
            String::from_utf8(output).unwrap().contains("ledger is valid"),
            "Run must wire the real FileLedger through to Report_Validation"
        );
    }

    /// A scratch ledger directory, real enough for `Run` to open with `FILE_SYSTEM`.
    ///
    /// Named with the process id so two suites running at once do not collide, and cleared
    /// first in case a previous run of this same test was killed before its own cleanup ran.
    fn Scratch_Directory(name: &str) -> std::path::PathBuf
    {
        let root = std::env::temp_dir().join(format!(
            "nomos-work-run-test-{name}-{}",
            std::process::id()
        ));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");
        std::fs::write(
            root.join("ledger.json"),
            format!(
                "{{\"schema_version\":{},\"items\":[]}}",
                nomos_ledger::SCHEMA_VERSION
            ),
        )
        .expect("a scratch ledger");
        return root;
    }
}
