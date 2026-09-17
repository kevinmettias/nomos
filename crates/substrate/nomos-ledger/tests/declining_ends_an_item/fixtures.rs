//! The fixtures every test in this suite is written against.
//!
//! One place to say what an item, a clock and a board are, so that a test reads as the claim
//! it makes rather than as the fixture it needs. A suite whose fixtures are restated per file
//! drifts into several boards that agree only by coincidence.

pub(crate) use nomos_ledger::{
    ClaimRefusal, ExclusionLedger, FileLedger, Holder, ItemId, ItemKind, ItemOrigin, ItemState,
    LedgerDocument, LedgerItem, ReleaseOutcome, SCHEMA_VERSION, VerificationRecord,
};
pub(crate) use nomos_platform::{
    Clock, DeterminismStrength, ReproducibilityScope, Strategy, Timestamp, TraceEquivalence,
};
pub(crate) use nomos_platform_std::{FileLock, StdFileSystem};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::time::Duration;

pub(crate) const NOW: i64 = 1_000_000;

pub(crate) const REASON: &str =
    "superseded by T-2, which landed the whole of this item's done_when and was verified first";

/// Held still, so that "when it was declined" is arithmetic rather than a sleep.
pub(crate) struct FixedClock(pub(crate) i64);

/// Fixed instants, so both the values and their timing reproduce.
impl Strategy for FixedClock
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Clock for &FixedClock
{
    fn Now(&self) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(self.0);
    }
}

pub(crate) fn Timestamp_From_Seconds(seconds: i64) -> Timestamp
{
    return Timestamp::From_Unix_Seconds(seconds);
}

pub(crate) fn Item_Reserving_Files(id: &str, files: &[&str]) -> LedgerItem
{
    use nomos_ledger::Territory as ItemTerritory;

    return LedgerItem {
        id: ItemId::New(id),
        title: format!("work item {id}"),
        why: "it needs doing".to_owned(),
        done_when: "the tests pass".to_owned(),
        kind: ItemKind::Correction,
        origin: ItemOrigin::Proposed,
        territory: ItemTerritory::Of_Files(files.iter().copied()),
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: None,
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        widened: Vec::new(),
        declined: None,
    };
}

/// A `Done` item, built rather than finished.
///
/// Running a real predicate would make this file about `finish`, which `gate_covers_finish.rs`
/// already is. What matters here is only that the state is `Done` and validation accepts it,
/// which needs the verification record store validation requires beside that state.
pub(crate) fn Finished_Item(id: &str, files: &[&str]) -> LedgerItem
{
    let mut item = Item_Reserving_Files(id, files);
    item.state = ItemState::Done;
    item.verified = Some(VerificationRecord {
        argv: vec!["cargo".to_owned(), "--version".to_owned()],
        exit_code: 0,
        output_tail: String::new(),
        verified_at: Timestamp_From_Seconds(NOW),
        gate: None,
        revision: None,
    });
    return item;
}

fn Document_Holding_Items(items: Vec<LedgerItem>) -> LedgerDocument
{
    return LedgerDocument {
        schema_version: SCHEMA_VERSION,
        items,
    };
}

pub(crate) fn Temporary_Directory(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-decline-{name}-{}", std::process::id()));
    Remove_Scratch(&path);
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    return path;
}

/// Removes a scratch directory this test made, reporting a failure rather than discarding it.
///
/// A directory that outlives its test is one the next run inherits, and a failed removal is
/// the only thing that would have said so -- which is why the failure is printed rather than
/// dropped.
pub(crate) fn Remove_Scratch(directory: &Path)
{
    if let Err(error) = std::fs::remove_dir_all(directory)
    {
        eprintln!("could not remove the scratch directory {}: {error}", directory.display());
    }
}

/// The clock every test that does not move time shares.
///
/// A `'static` clock is what lets [`Board_At`] hand back a ledger: the ledger borrows its
/// clock, so a local one could not outlive the call that built it.
static AT_NOW: FixedClock = FixedClock(NOW);

/// The one-hour lease every test here takes, said once.
const LEASE: Duration = Duration::from_secs(3_600);

/// The ledger every test here builds, named once so the verbs below can take it.
pub(crate) type Board = FileLedger<StdFileSystem, &'static FixedClock, FileLock>;

/// A ledger on a fresh temporary directory, and the directory it lives on.
pub(crate) struct BoardOnDisk
{
    pub(crate) directory: PathBuf,
    pub(crate) ledger: Board,
}

/// A ledger on a fresh temporary directory, already holding the board it starts from.
pub(crate) fn Board_At(name: &str, items: Vec<LedgerItem>) -> BoardOnDisk
{
    let directory = Temporary_Directory(name);
    let ledger = Ledger_At(&directory, &AT_NOW);
    ledger.Save(&Document_Holding_Items(items)).expect("a fresh ledger is valid");

    return BoardOnDisk { directory, ledger };
}

/// One agent claims one item for the standard lease, and it is expected to succeed.
pub(crate) fn Claim_For_Holder<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &Holder<'_>)
{
    ledger
        .Claim(&ItemId::New(item), holder.As_Text(), LEASE)
        .unwrap_or_else(|refusal| {
            panic!("the fixture claim was refused: {}", refusal.Describe())
        });
}

/// A claim that is expected to be refused, with the refusal handed back as the value the
/// test is about.
pub(crate) fn Refusal_From_Claim(ledger: &mut Board, item: &str, holder: &Holder<'_>) -> ClaimRefusal
{
    return ledger
        .Claim(&ItemId::New(item), holder.As_Text(), LEASE)
        .expect_err("this claim is contended and must be refused");
}

/// An item ended with the standard reason, which is expected to succeed.
pub(crate) fn Decline_Item_For_Reason(ledger: &mut Board, item: &str, holder: &Holder<'_>)
{
    ledger
        .Decline(&ItemId::New(item), holder.As_Text(), REASON)
        .expect("an unclaimed item is the case this verb exists for");
}

/// A decline expected to be refused, with the refusal handed back as the value the test is
/// about.
pub(crate) fn Decline_Refused(ledger: &mut Board, item: &str, holder: &Holder<'_>, reason: &str) -> ClaimRefusal
{
    return ledger
        .Decline(&ItemId::New(item), holder.As_Text(), reason)
        .expect_err("this decline is contended and must be refused");
}

/// A holder giving up its own claim, which is the remedy every refusal here names.
pub(crate) fn Give_Up(ledger: &mut Board, item: &str, holder: &Holder<'_>, reason: &str)
{
    ledger
        .Release(&ItemId::New(item), holder.As_Text(), ReleaseOutcome::Abandoned {
            reason: reason.to_owned(),
        })
        .expect("a holder may give up its own claim");
}

/// The board's only item, read back off disk.
pub(crate) fn Only_Item(ledger: &Board) -> LedgerItem
{
    let after = ledger
        .Load()
        .expect("the fixture wrote this board and every change since went through a verb");

    return after.items.into_iter().next().expect("the item survives");
}

fn Ledger_At<'clock>(
    directory: &Path,
    clock: &'clock FixedClock,
) -> FileLedger<StdFileSystem, &'clock FixedClock, FileLock>
{
    return FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        clock,
        FileLock::At(directory.join("ledger.lock")),
    );
}
