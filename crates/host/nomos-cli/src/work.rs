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
pub(crate) use nomos_work_orchestration::{ClaimRequest, EndingRequest, WorkCommand};

use listing::{
    Listed_As, Listing_Label, Nothing_Listed, Print_Claim, Print_History, Print_Listing,
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

    return Render_Outcome(command, outcome, output);
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
        std::fs::create_dir_all(root.join(RECORD_DIRECTORY)).unwrap();
        std::fs::write(root.join(RECORD_DIRECTORY).join("OD-EXAMPLE-001.md"), "# example").unwrap();
        std::fs::write(root.join(RECORD_DIRECTORY).join("OD-EXAMPLE-002.md"), "# example").unwrap();

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
/// shape the command it is paired with produces. The `_ => unreachable!()` arm exists only
/// because Rust cannot see that invariant from the two enums' shapes alone —
/// `nomos_work_orchestration::Run` always returns the one `WorkOutcome` variant naming the
/// `WorkCommand` variant it was given.
fn Render_Outcome(command: &WorkCommand, outcome: WorkOutcome, output: &mut impl std::io::Write) -> ExitCode
{
    return match (command, outcome)
    {
        (WorkCommand::List { state }, WorkOutcome::List(result)) =>
        {
            Render_List(state.as_deref(), result, output)
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
        (WorkCommand::Audit, WorkOutcome::Audit(result)) => Render_Audit(result, output),
        // rust-panic: allow: Run is the only caller of this match, and it always builds
        // WorkOutcome from the same WorkCommand variant it dispatched on -- no other pairing
        // reaches this function.
        (_, _) => unreachable!("Run always pairs a command with its own outcome shape"),
    };
}

fn Render_List(
    state: Option<&str>,
    result: Result<BoardView, LedgerError>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let BoardView { document, now } = match result
    {
        Ok(view) => view,
        Err(error) => return Report_Error(&error, output),
    };

    Print_Board(&document, state, now, output);

    return ExitCode::Ok;
}

/// Every item in `state`'s bucket, or the "nothing" line when none matched -- and, on the
/// unfiltered board only, the one line naming which item to claim next.
fn Print_Board(
    document: &LedgerDocument,
    state: Option<&str>,
    now: nomos_platform::Timestamp,
    output: &mut impl std::io::Write,
)
{
    let mut shown = 0_u32;
    for item in &document.items
    {
        let Some(label) = Listed_As(document, item, state, now)
        else
        {
            continue;
        };
        Print_Listing(item, label, output);
        shown = shown.saturating_add(1);
    }
    if shown == 0
    {
        Nothing_Listed(state, output);
    }
    // Only on the unfiltered board. `--state` asks for one bucket's rows, and a summary
    // naming an item outside that bucket would contradict the very filter the caller asked
    // for -- `--state lapsed` is not the place to also learn what is `Ready` elsewhere.
    if state.is_none()
    {
        Print_Next(document, now, output);
    }
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

/// Reports one item, including what has happened to it.
///
/// `list` is one line per item and cannot carry prose. That is why an abandonment had
/// nowhere to be read even once the ledger began keeping one: a record no surface reports
/// is a record only somebody willing to read the JSON can find, which is most of the way
/// back to not keeping it.
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

fn Render_Audit(result: Result<BoardView, LedgerError>, output: &mut impl std::io::Write) -> ExitCode
{
    let BoardView { document, now } = match result
    {
        Ok(view) => view,
        Err(error) => return Report_Error(&error, output),
    };

    Print_Audit(&document, now, output);

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
