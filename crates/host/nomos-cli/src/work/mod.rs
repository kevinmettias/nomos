//! `nomos work` — the ledger, from a terminal.

use nomos_ledger::{
    ExclusionLedger, FileLedger, Finish, Finishing, ItemId, LedgerItem, ReleaseOutcome, Territory,
};
use nomos_platform::Clock;
use nomos_platform_std::{FileLock, StdFileSystem, StdProcessLauncher, SystemClock};
use std::path::Path;
use std::time::Duration;

mod parse;
mod report;
#[cfg(test)]
mod tests;

pub use parse::Parse;

use report::{
    Amendment_Note, Blocking_Refusal, Code_For_Refusal, Listed_As, Listing_Label, Nothing_Listed,
    Print_Blocked, Print_Claim, Print_History, Print_Listing, Report_Claim, Report_Decline,
    Report_Error, Report_Finish, Report_Release, Report_Validation,
};

/// What the process exits with.
///
/// A contract, not an implementation detail. Agents branch on these rather than parsing
/// output, so they are documented here and covered by tests. The distinction that earns
/// its own code is [`ExitCode::ClaimUnavailable`]: an agent that is told the item is
/// taken should try another one, and an agent that is told the ledger is broken should
/// stop and get a human — collapsing those into "non-zero" makes the first case
/// indistinguishable from the second.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExitCode
{
    /// The operation succeeded.
    Ok = 0,
    /// The ledger is invalid, or an operation was refused on its merits.
    ValidationError = 1,
    /// The command line was wrong.
    Usage = 2,
    /// Somebody else holds it. Retryable.
    ClaimUnavailable = 3,
    /// A conflict a human has to resolve.
    Conflict = 4,
    /// The ledger or its lock could not be used at all.
    StoreError = 5,
}

impl ExitCode
{
    /// The numeric code.
    #[must_use]
    pub const fn Value(self) -> i32
    {
        return self as i32;
    }
}

/// What to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkCommand
{
    /// Show items, optionally filtered by state.
    List
    {
        /// Only items in this state.
        state: Option<String>,
    },
    /// Report one item, including what has happened to it.
    Show
    {
        /// Which item.
        item: ItemId,
    },
    /// Put a new item on the ledger.
    Add
    {
        /// The item to record.
        item: Box<LedgerItem>,
        /// Which of the records it reserves the item edits rather than allocates.
        ///
        /// Beside the item rather than on it. The declaration decides whether the add is
        /// refused and has no reader afterwards, so carrying it on [`LedgerItem`] would put a
        /// field on a document two sessions share — and a build older than a field drops it
        /// silently at exit 0, which is what `OD-LEDGER-008` prices.
        amending: Territory,
    },
    /// Run an item's verification predicate and record it done if it passes.
    Finish
    {
        /// Which item.
        item: ItemId,
        /// Who holds it.
        holder: String,
    },
    /// Take an item.
    Claim(ClaimRequest),
    /// Extend a held claim.
    Renew(ClaimRequest),
    /// Take over an item whose holder's lease ran out, keeping the claim it displaces.
    ///
    /// Separate from [`WorkCommand::Claim`] because taking another agent's abandoned work is
    /// a decision, and a decision belongs in a verb somebody typed — `OD-LEDGER-012`.
    TakeOver(ClaimRequest),
    /// Give up a claim without finishing.
    Abandon(EndingRequest),
    /// End an item that turned out not to be work.
    ///
    /// Separate from [`WorkCommand::Abandon`] because they are about different subjects —
    /// abandoning ends a claim and puts the item back on the board, declining ends the item —
    /// and because the items this exists for are unclaimed, which `abandon` cannot reach.
    /// `OD-LEDGER-019`.
    Decline(EndingRequest),
    /// Check the ledger's invariants.
    Validate,
    /// Show what would block a claim.
    Audit,
}

/// Who is holding what, and for how long.
///
/// One type for `claim`, `renew` and `takeover` because they take exactly the same three
/// arguments and default the lease the same way. Three identical types would be three
/// chances for them to drift apart in what they accept while being documented as identical.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ClaimRequest
{
    /// Which item.
    pub item: ItemId,
    /// Who is taking or holding it.
    pub holder: String,
    /// How long to hold it.
    pub lease: Duration,
}

/// Who is ending what, and why.
///
/// Shared by `abandon` and `decline` for the shape of the argument list only. What they end
/// is different, which is why they stay two verbs and two ledger calls — `OD-LEDGER-019`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EndingRequest
{
    /// Which item.
    pub item: ItemId,
    /// Who is ending it.
    pub holder: String,
    /// Why.
    pub reason: String,
}

/// Runs a command against the ledger at `directory`, writing to `output`.
///
/// Returns the exit code rather than exiting, so the whole surface is testable.
pub fn Run(
    command: &WorkCommand,
    directory: &Path,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let mut ledger = FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        SystemClock,
        FileLock::At(directory.join("ledger.lock")),
    );

    return match command
    {
        WorkCommand::List { state } => List(&ledger, state.as_deref(), output),
        WorkCommand::Show { item } => Show(&ledger, item, output),
        WorkCommand::Add { item, amending } =>
        {
            let published = Published_Records(directory);

            Add(&mut ledger, item, Declared {
                published: &published,
                amending,
            }, output)
        }
        WorkCommand::Finish { item, holder } => Finished(&mut ledger, item, holder, output),
        WorkCommand::Claim(request) => Claimed(&mut ledger, request, output),
        WorkCommand::Renew(request) => Renewed(&mut ledger, request, output),
        WorkCommand::TakeOver(request) => Taken_Over(&mut ledger, request, output),
        WorkCommand::Abandon(request) => Abandoned(&mut ledger, request, output),
        WorkCommand::Decline(request) => Declined(&mut ledger, request, output),
        WorkCommand::Validate => Report_Validation(&ledger, output),
        WorkCommand::Audit => Audit(&ledger, output),
    };
}

/// Runs the item's predicate and reports what it said.
///
/// No working directory: the predicate runs where the user invoked `nomos`, which for a
/// repository tool run inside a repository is the repository. A predicate silently
/// relocated into `work/` would fail in ways that look like the work being wrong.
fn Finished(
    ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>,
    item: &ItemId,
    holder: &str,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let finishing = Finishing { item, holder };
    let outcome = Finish(ledger, &StdProcessLauncher, &finishing, None);

    return Report_Finish(outcome, output);
}

/// One `Report_Claim` across the three reservation verbs, which is the point: one mapping
/// from a refusal to an exit code, so `claim`, `renew` and `takeover` cannot come to
/// disagree about what a refusal means, and the README's table does not move.
fn Claimed(
    ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>,
    request: &ClaimRequest,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let outcome = ledger.Claim(&request.item, &request.holder, request.lease);

    return Report_Claim(outcome, output);
}

fn Renewed(
    ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>,
    request: &ClaimRequest,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let outcome = ledger.Renew(&request.item, &request.holder, request.lease);

    return Report_Claim(outcome, output);
}

fn Taken_Over(
    ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>,
    request: &ClaimRequest,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let outcome = ledger.Take_Over(&request.item, &request.holder, request.lease);

    return Report_Claim(outcome, output);
}

fn Abandoned(
    ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>,
    request: &EndingRequest,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let abandoned = ReleaseOutcome::Abandoned {
        reason: request.reason.clone(),
    };
    let outcome = ledger.Release(&request.item, &request.holder, abandoned);

    return Report_Release(outcome, output);
}

fn Declined(
    ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>,
    request: &EndingRequest,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let outcome = ledger.Decline(&request.item, &request.holder, &request.reason);

    return Report_Decline(&request.item, outcome, output);
}

fn List(
    ledger: &FileLedger<StdFileSystem, SystemClock, FileLock>,
    state: Option<&str>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let document = match ledger.Load()
    {
        Ok(document) => document,
        Err(error) => return Report_Error(&error, output),
    };
    let now = SystemClock.Now();
    let mut shown = 0_u32;
    for item in &document.items
    {
        let Some(label) = Listed_As(&document, item, state, now)
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

    return ExitCode::Ok;
}

/// Reports one item, including what has happened to it.
///
/// `list` is one line per item and cannot carry prose. That is why an abandonment had
/// nowhere to be read even once the ledger began keeping one: a record no surface reports
/// is a record only somebody willing to read the JSON can find, which is most of the way
/// back to not keeping it.
fn Show(
    ledger: &FileLedger<StdFileSystem, SystemClock, FileLock>,
    item: &ItemId,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let document = match ledger.Load()
    {
        Ok(document) => document,
        Err(error) => return Report_Error(&error, output),
    };

    let Some(found) = document.items.iter().find(|candidate| return &candidate.id == item)
    else
    {
        let _ = writeln!(output, "no item named {item}");
        return ExitCode::Conflict;
    };
    let now = SystemClock.Now();
    let label = Listing_Label(&document, found, now);

    let _ = writeln!(output, "{} {}", found.id, found.title);
    let _ = writeln!(output, "state: {label}");
    Print_Claim(found, now, output);
    Print_History(found, output);

    return ExitCode::Ok;
}

/// Records a new item, refusing a duplicate identifier and a document that would not
/// validate. It does **not** refuse an item whose territory somebody already holds.
///
/// The whole document is validated before the write, so an item that would break an
/// invariant never lands. The alternative — write now, notice later — leaves every agent
/// reading a ledger the system itself says is wrong.
///
/// # The control this used to claim
///
/// This comment said "refusing one whose territory is already spoken for" from the commit
/// that wrote the command until `OD-LEDGER-010`, and nothing here ever did that.
/// [`nomos_ledger::Validate`] compares territories only between items holding an *active
/// claim*; an item arriving here holds none, so the comparison has nothing to say about it
/// and never fires.
///
/// The behaviour is the one to keep and the sentence is what moved. Opening an item on
/// ground somebody holds is how this board is used — `work/ledger.json` is in nobody's
/// territory precisely so that `add` stays available when every item on the board is held,
/// and a refusal here would shut the one door that is open when the board is fully
/// claimed. Exclusion belongs to `claim`, which is where an agent is about to edit files.
/// `add` writes a sentence about work that may not start for days.
///
/// What is guaranteed is therefore smaller than the old sentence promised, and it is
/// stated exactly because the promise is what the next reader will act on: the identifier
/// is unused, and the document that results still satisfies its own invariants. Nothing
/// here says two agents may edit one file, and nothing here is what stops them.
///
/// # The exclusion this *does* need, and did not have
///
/// Both of those guarantees were until `OD-LEDGER-021` written outside the lock. This
/// function read the whole document, decided against it and wrote the whole document back
/// with nothing held in between — the shape `OD-LEDGER-015` removed from `Claim`, `Renew`
/// and `Release`, left behind here because those three were fixed by name. An add that read
/// the board before a concurrent verb's write put its own snapshot back over it, and the
/// caller was told exit 0 either way.
///
/// So the read, the duplicate check and the write are now one [`FileLedger::Add`], and what
/// is left here is reporting. The check in particular had to travel with them: outside the
/// lock, two sessions adding one identifier both read a board without it, and the second
/// write produced a document `Validate` calls invalid — a board that then refuses to load,
/// for two callers who were each told they had succeeded.
/// Every record this repository has already published, as repository-relative paths.
///
/// Read here rather than in the store, and that division is the point rather than a
/// convenience. `OD-LEDGER-021` put the *decision* behind the ledger's lock and left the
/// command layer its input and its reporting; enumerating a repository is input. A general
/// exclusion ledger that learned to walk a source tree would be answering a question about
/// this repository's conventions, and `nomos-ledger` already stretches as far as it should
/// by knowing what a record filename folds to.
///
/// An unreadable or absent directory yields nothing rather than refusing. That is the one
/// judgement here worth stating, because this repository's usual rule is the opposite: a
/// check that cannot find its subject must fail loudly. It does not apply, because this is
/// not the check — the open-item comparison still runs, and it is the half that races. A
/// tree with no `docs/records` is a ledger being used somewhere that has no records, and
/// refusing every `add` in it would be this repository's convention refusing everybody
/// else's.
fn Published_Records(directory: &Path) -> Territory
{
    // The ledger lives in `work/`, so the repository is its parent. A `work/` at the root of
    // nothing has no records, which the walk below reports as none.
    let Some(root) = directory.parent()
    else
    {
        return Territory::Empty();
    };

    let mut published = Record_Files(root);
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
fn Record_Files(root: &Path) -> Vec<String>
{
    let Ok(entries) = std::fs::read_dir(root.join(RECORD_DIRECTORY))
    else
    {
        return Vec::new();
    };

    let mut published = Vec::new();
    for entry in entries.flatten()
    {
        if let Some(name) = entry.file_name().to_str()
        {
            published.push(format!("{RECORD_DIRECTORY}/{name}"));
        }
    }

    return published;
}

/// Where this repository authors its decision records, relative to the repository root.
const RECORD_DIRECTORY: &str = "docs/records";

/// The two territories an `add` declares beside the item's own: what the repository has
/// already published, and what this item reserves in order to amend.
#[derive(Clone, Copy)]
struct Declared<'a>
{
    published: &'a Territory,
    amending: &'a Territory,
}

fn Add(
    ledger: &mut FileLedger<StdFileSystem, SystemClock, FileLock>,
    item: &LedgerItem,
    declared: Declared<'_>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    let Declared { published, amending } = declared;
    // The lock's holder name is a courtesy for a stale-takeover report and never an
    // identity that is checked, which is why `add` can name itself here while every other
    // verb passes the agent that asked. `add` takes no `--holder` because it takes no
    // claim: the item it writes is `Ready` and belongs to nobody yet.
    return match ledger.Add(item, "nomos work add", published, amending)
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

fn Audit(
    ledger: &FileLedger<StdFileSystem, SystemClock, FileLock>,
    output: &mut impl std::io::Write,
) -> ExitCode
{
    // Once, for the whole report. `Conflicts` loads and parses the ledger internally, so
    // asking it per item read the file once per item — sixty opens to answer one question
    // about sixty items, and sixty chances for the answer to be about a different board
    // than the line above it.
    let document = match ledger.Load()
    {
        Ok(document) => document,
        Err(error) => return Report_Error(&error, output),
    };
    let now = SystemClock.Now();
    let mut reported = 0_u32;
    for item in &document.items
    {
        let Some(refusal) = Blocking_Refusal(&document, item, now)
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

    return ExitCode::Ok;
}
