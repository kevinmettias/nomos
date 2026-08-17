//! The board every test in this suite is written against.
//!
//! One place to say what an item, a territory, a holder and a clock are, so that a test
//! reads as the claim it makes rather than as the fixture it needs. A suite whose fixtures
//! are restated per file drifts into several boards that agree only by coincidence.

// PROBE


pub(crate) use nomos_ledger::{
    Finishing,
    Abandonment, AddRefusal, Blocker, Claim, ClaimRefusal, Declination, ExclusionLedger, FileLedger, Finish,
    FinishRefusal, GateOutcome, ItemId, ItemState, LedgerDocument, LedgerError, LedgerItem,
    ReleaseOutcome, Reservation, SCHEMA_VERSION, Territory as ItemTerritory, Validate, VerificationPredicate,
    VerificationRecord,
};
pub(crate) use nomos_model::SetResolution;
pub(crate) use nomos_platform::{Clock, FileSystem, FileSystemError, Timestamp};
pub(crate) use nomos_platform_std::{FileLock, StdFileSystem, StdProcessLauncher};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::sync::atomic::{AtomicBool, Ordering};
pub(crate) use std::sync::{Condvar, Mutex};
pub(crate) use std::thread::ThreadId;
pub(crate) use std::time::Duration;

/// A clock the tests hold still, so lease expiry is reached by arithmetic rather than by
/// sleeping. A suite that sleeps to reach a deadline is a suite that is slow and
/// intermittently wrong.
pub(crate) struct FixedClock(pub(crate) i64);

impl Clock for FixedClock
{
    fn Now(&self) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(self.0);
    }
}

/// Offered by reference as well, so the two shared statics can be lent to many ledgers while
/// a test that needs its own moment hands one over by value.
impl Clock for &FixedClock
{
    fn Now(&self) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(self.0);
    }
}

pub(crate) const NOW: i64 = 1_000_000;

pub(crate) fn At(seconds: i64) -> Timestamp
{
    return Timestamp::From_Unix_Seconds(seconds);
}

pub(crate) fn Territory(files: &[&str]) -> ItemTerritory
{
    return ItemTerritory::Of_Files(files.iter().copied());
}

pub(crate) fn Item(id: &str, files: &[&str]) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: format!("work item {id}"),
        why: "it needs doing".to_owned(),
        done_when: "the tests pass".to_owned(),
        territory: Territory(files),
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: None,
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        declined: None,
    };
}

pub(crate) fn Held_By(mut item: LedgerItem, holder: &str, expires: i64) -> LedgerItem
{
    item.state = ItemState::Claimed;
    item.claim = Some(Claim {
        holder: holder.to_owned(),
        acquired_at: At(NOW),
        lease_expires_at: At(expires),
    });
    return item;
}

/// `SCHEMA_VERSION` rather than a literal `1`.
///
/// `Save` stamps what it writes with the version this build understands, so a fixture holding a
/// literal would stop equalling its own reload the moment the constant moves — and
/// `Test_The_Ledger_Should_Round_Trip_Losslessly` would then fail for a reason that has nothing
/// to do with round-tripping.
pub(crate) fn Document(items: Vec<LedgerItem>) -> LedgerDocument
{
    return LedgerDocument {
        schema_version: SCHEMA_VERSION,
        items,
    };
}

/// A temporary repository that removes itself when the test holding it ends.
///
/// Every test here used to close with its own `remove_dir_all`, which is a line that only
/// runs when the test passes: a failed assertion unwinds straight past it. `Drop` runs on
/// the unwind too, so the tree is cleared exactly when the value goes out of scope and the
/// cleanup is no longer a step a test can forget or an assertion can skip.
pub(crate) struct Scratch(pub(crate) PathBuf);

impl Drop for Scratch
{
    fn drop(&mut self)
    {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for Scratch
{
    type Target = Path;

    fn deref(&self) -> &Path
    {
        return &self.0;
    }
}

pub(crate) fn Temp_Dir(name: &str) -> Scratch
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-ledger-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    Write_Gate(&path);

    return Scratch(path);
}

/// Every tree these tests build is a repository with a gate, because finishing now reads
/// one and refuses when it cannot.
///
/// The lint step is `cargo --version` rather than the real clippy invocation. These tests
/// are about what a *predicate's* exit code does to an item; running a real workspace lint
/// in each of them would make the suite take minutes and would couple it to whatever the
/// workspace currently contains. What the derived step actually is, and that it comes from
/// the workflow rather than from a constant, is covered in `gate_covers_finish.rs`.
pub(crate) fn Write_Gate(directory: &Path)
{
    let workflows = directory.join(".github").join("workflows");
    std::fs::create_dir_all(&workflows).expect("test needs a workflow directory");
    std::fs::write(
        workflows.join("gate.yml"),
        "jobs:\n\
         \x20 gate:\n\
         \x20   steps:\n\
         \x20     - name: Lint\n\
         \x20       run: cargo --version\n",
    )
    .expect("test needs a workflow");
}

/// The clock every test that does not move time shares.
///
/// A `'static` clock is what lets [`Board_At`] hand back a ledger: the ledger borrows its
/// clock, so a local one could not outlive the call that built it. A test that needs time
/// to move builds its own later clock and a second ledger over the same directory.
pub(crate) static AT_NOW: FixedClock = FixedClock(NOW);

/// The one-hour lease every test here takes, said once.
pub(crate) const LEASE: Duration = Duration::from_secs(3_600);

/// One agent claims one item for the standard lease, and it is expected to succeed.
///
/// A refusal here is the fixture failing rather than the assertion under test, so it panics
/// with the refusal's own words instead of returning it.
pub(crate) fn Take<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &str)
{
    ledger
        .Claim(&ItemId::New(item), holder, LEASE)
        .unwrap_or_else(|refusal| panic!("the fixture claim was refused: {}", refusal.Describe()));
}

/// An item whose territory carries a pattern, which nothing on the command line can build
/// any more — `OD-LEDGER-013` withdrew `--territory-pattern` — and which these two tests
/// still construct by hand, because the state stays reachable by editing the document.
pub(crate) fn Patterned(id: &str, files: &[&str], pattern: &str) -> LedgerItem
{
    let mut item = Item(id, files);
    item.territory = item.territory.With_Pattern(pattern);

    return item;
}

/// An item that is `Done` and carries the evidence that made it done.
///
/// A `Done` item without a verification record is not a valid ledger, so the two are built
/// together or not at all.
pub(crate) fn Finished(id: &str, files: &[&str]) -> LedgerItem
{
    let mut item = Item(id, files);
    item.state = ItemState::Done;
    item.verified = Some(VerificationRecord {
        argv: vec!["cargo".to_owned(), "test".to_owned()],
        exit_code: 0,
        output_tail: "ok".to_owned(),
        verified_at: At(NOW),
        gate: None,
        revision: None,
    });

    return item;
}

/// An item that will never be done — declined, with the reason it carries.
pub(crate) fn Declined(id: &str, files: &[&str], reason: &str) -> LedgerItem
{
    let mut item = Item(id, files);
    item.state = ItemState::Declined {
        reason: reason.to_owned(),
    };

    return item;
}

/// A holder releases its own claim as finished, carrying the evidence that made it so.
///
/// The record is the fixture rather than the subject — what these tests assert is what the
/// store does with it — so building it here keeps eleven lines of literal out of the test.
pub(crate) fn Release_As_Finished<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &str)
{
    ledger
        .Release(
            &ItemId::New(item),
            holder,
            ReleaseOutcome::Finished(VerificationRecord {
                argv: vec!["cargo".to_owned(), "test".to_owned()],
                exit_code: 0,
                output_tail: "ok".to_owned(),
                verified_at: At(NOW),
                gate: None,
                revision: None,
            }),
        )
        .expect("a release carrying evidence must be accepted");
}

/// Every abandonment the item kept, as who stopped and what they said, oldest first.
pub(crate) fn Abandonments(item: &LedgerItem) -> Vec<(&str, &str)>
{
    return item
        .abandoned
        .iter()
        .map(|entry| return (entry.holder.as_str(), entry.reason.as_str()))
        .collect();
}

/// A holder gives up its own claim, with the words it gave for stopping.
pub(crate) fn Abandon<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &str, reason: &str)
{
    ledger
        .Release(
            &ItemId::New(item),
            holder,
            ReleaseOutcome::Abandoned {
                reason: reason.to_owned(),
            },
        )
        .expect("a holder may give up its own claim");
}

/// A claim that is expected to be refused, with the refusal handed back as the value the
/// test is about.
pub(crate) fn Refused<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &str) -> ClaimRefusal
{
    return ledger
        .Claim(&ItemId::New(item), holder, LEASE)
        .expect_err("this claim is contended and must be refused");
}

/// Runs an item's own verification predicate through the ledger, in a named repository.
pub(crate) fn Finish_In(
    ledger: &mut FileLedger<StdFileSystem, &FixedClock, FileLock>,
    directory: &Path,
    item: &str,
    holder: &str,
) -> Result<VerificationRecord, FinishRefusal>
{
    return Finish(
        ledger,
        &StdProcessLauncher,
        &Finishing {
            item: &ItemId::New(item),
            holder,
        },
        Some(directory),
    );
}

/// The one item a single-item board carries, read back through the file.
///
/// Nearly every test below reaches the same pair of lines to get at the one value it asserts
/// on. Naming the pair keeps the load out of the assertion's way, and going through the file
/// rather than through the in-memory value is the point of asserting at all: what the next
/// session sees is what was written, not what this one still holds.
pub(crate) fn Only_Item<Clock: nomos_platform::Clock>(
    ledger: &FileLedger<StdFileSystem, Clock, FileLock>,
) -> LedgerItem
{
    return ledger
        .Load()
        .expect("the ledger is readable")
        .items
        .into_iter()
        .next()
        .expect("the item survives");
}

/// One named item on a board carrying several, read back through the file.
///
/// The multi-item boards assert about one of their items and use the others as the context
/// that makes the assertion mean something, so reaching the subject by name rather than by
/// position keeps the test honest when the board is reordered.
pub(crate) fn Named<Clock: nomos_platform::Clock>(
    id: &str,
    ledger: &FileLedger<StdFileSystem, Clock, FileLock>,
) -> LedgerItem
{
    return ledger
        .Load()
        .expect("the ledger is readable")
        .items
        .into_iter()
        .find(|item| return item.id == ItemId::New(id))
        .unwrap_or_else(|| panic!("{id} survives"));
}

/// Who an item's own claim names, or nothing when it carries none.
///
/// The tests compare holders, and `claim.as_ref().map(|claim| claim.holder.clone())` is four
/// tokens of plumbing in front of one word. This says the word.
pub(crate) fn Holder(item: &LedgerItem) -> Option<&str>
{
    return item.claim.as_ref().map(|claim| return claim.holder.as_str());
}

/// Who holds an item now and every holder a takeover displaced, as one value.
///
/// The takeover tests all turn on the *pair*: a takeover that installs the new holder while
/// dropping the old one passes an assertion on either field alone, and is exactly the outcome
/// `OD-LEDGER-012` exists to prevent. Comparing the pair is what makes that one failure.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Standing<'a>
{
    pub(crate) held_by: Option<&'a str>,
    pub(crate) displaced: Vec<&'a str>,
}

pub(crate) fn Standing_Of(item: &LedgerItem) -> Standing<'_>
{
    return Standing {
        held_by: Holder(item),
        displaced: item
            .displaced
            .iter()
            .map(|claim| return claim.holder.as_str())
            .collect(),
    };
}

/// Two hours after [`AT_NOW`], by which time the one-hour lease these tests take has lapsed.
pub(crate) static AT_LATER: FixedClock = FixedClock(NOW + 7_200);

/// The same board read again once its lease has lapsed.
pub(crate) fn After_The_Lapse(
    directory: &Path,
) -> FileLedger<StdFileSystem, &'static FixedClock, FileLock>
{
    return Ledger_At(directory, &AT_LATER);
}

/// One agent takes a lapsed item over for the standard lease, with the verdict handed back.
///
/// Unlike [`Take`] this returns rather than panics, because both outcomes are subjects here:
/// half these tests are about the takeover succeeding and half about it being refused.
pub(crate) fn Take_Over_In<Clock: nomos_platform::Clock>(
    ledger: &mut FileLedger<StdFileSystem, Clock, FileLock>,
    item: &str,
    holder: &str,
) -> Result<Reservation, ClaimRefusal>
{
    return ledger.Take_Over(&ItemId::New(item), holder, LEASE);
}

/// The same board read again by a ledger standing at a named moment.
///
/// A ledger that owns its clock is what makes a moment one argument. Borrowing one forces
/// every test that moves time to bind the clock first and keep it alive by hand, which is two
/// lines of scaffolding in front of the one number the test is actually varying.
pub(crate) fn Ledger_When(directory: &Path, seconds: i64) -> FileLedger<StdFileSystem, FixedClock, FileLock>
{
    return Ledger_At(directory, FixedClock(seconds));
}

/// A ledger on a fresh temporary directory, already holding the board it starts from.
pub(crate) fn Board_At(
    name: &str,
    items: Vec<LedgerItem>,
) -> (Scratch, FileLedger<StdFileSystem, &'static FixedClock, FileLock>)
{
    let directory = Temp_Dir(name);
    let ledger = Ledger_At(&directory, &AT_NOW);
    ledger.Save(&Document(items)).expect("a fresh ledger is valid");

    return (directory, ledger);
}

pub(crate) fn Ledger_At<Clock: nomos_platform::Clock>(
    directory: &Path,
    clock: Clock,
) -> FileLedger<StdFileSystem, Clock, FileLock>
{
    let ledger_path = directory.join("ledger.json");
    let lock_path = directory.join("ledger.lock");

    return FileLedger::At(
        ledger_path,
        StdFileSystem,
        clock,
        FileLock::At(lock_path),
    );
}


// ---------------------------------------------------------------------------
// Acceptance 1 — two active claims on overlapping territory are refused.
// ---------------------------------------------------------------------------

/// The reason these tests assert on.
///
/// Prose rather than a marker, and asserted as text rather than as presence. A field that
/// exists and holds an empty string satisfies `is_some()`, which is exactly the assertion
/// that would have let the old behaviour through.
pub(crate) const REASON: &str =
    "the fixture never reproduced the shape that broke it, so the control proved nothing";

/// A command that exits with the given code, on either platform family.
pub(crate) fn Exits_With(code: i32) -> Vec<String>
{
    return if cfg!(windows)
    {
        vec!["cmd".to_owned(), "/C".to_owned(), format!("exit {code}")]
    }
    else
    {
        vec!["sh".to_owned(), "-c".to_owned(), format!("exit {code}")]
    };
}

pub(crate) fn Item_Verified_By(id: &str, files: &[&str], argv: Vec<String>) -> LedgerItem
{
    let mut item = Item(id, files);
    item.verification = Some(VerificationPredicate::New(argv));
    return item;
}
