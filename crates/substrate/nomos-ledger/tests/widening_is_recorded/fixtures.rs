//! The fixtures every test in this suite is written against.
//!
//! One place to say what an item, a clock and a board are, so that a test reads as the claim
//! it makes rather than as the fixture it needs. A suite whose fixtures are restated per file
//! drifts into several boards that agree only by coincidence.

pub(crate) use nomos_ledger::{
    ClaimRefusal, ExclusionLedger, FileLedger, Holder, ItemId, ItemKind, ItemOrigin, ItemState,
    LedgerDocument, LedgerItem, SCHEMA_VERSION, Territory,
};
pub(crate) use nomos_platform::{
    Clock, FilesystemLock, DeterminismStrength, ReproducibilityScope, Strategy, Timestamp,
    TraceEquivalence,
};
pub(crate) use nomos_platform_std::{FileLock, StdFileSystem};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::time::Duration;

pub(crate) const NOW: i64 = 1_000_000;
pub(crate) const LEASE: Duration = Duration::from_secs(3_600);

/// Held still, so that "when it was widened" is arithmetic rather than a sleep.
pub(crate) struct FixedClock(pub(crate) i64);

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

/// The clock every test that does not move time shares.
///
/// A `'static` clock is what lets [`Board_At`] hand back a ledger: the ledger borrows its
/// clock, so a local one could not outlive the call that built it.
pub(crate) static AT_NOW: FixedClock = FixedClock(NOW);

/// Two hours on, by which time the one-hour lease every test here takes has lapsed.
static AT_LATER: FixedClock = FixedClock(NOW + 7_200);

/// The ledger every test here builds, named once so the helpers below can take it.
pub(crate) type Board = FileLedger<StdFileSystem, &'static FixedClock, FileLock>;

/// A ledger on a fresh temporary directory, and the directory it lives on.
pub(crate) struct BoardOnDisk
{
    pub(crate) directory: PathBuf,
    pub(crate) ledger: Board,
}

pub(crate) fn Item(id: &str, files: &[&str]) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: format!("work item {id}"),
        why: "it needs doing".to_owned(),
        done_when: "the tests pass".to_owned(),
        kind: ItemKind::Correction,
        origin: ItemOrigin::Proposed,
        territory: Territory::Of_Files(files.iter().copied()),
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

pub(crate) fn Temporary_Directory(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-widen-{name}-{}", std::process::id()));
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

pub(crate) fn Ledger_At<'clock>(
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

/// A ledger on a fresh temporary directory, already holding the board it starts from.
pub(crate) fn Board_At(name: &str, items: Vec<LedgerItem>) -> BoardOnDisk
{
    let directory = Temporary_Directory(name);
    let ledger = Ledger_At(&directory, &AT_NOW);
    ledger
        .Save(&LedgerDocument { schema_version: SCHEMA_VERSION, items })
        .expect("a fresh ledger is valid");

    return BoardOnDisk { directory, ledger };
}

/// The board of [`Board_At`], claimed by its holder and then reopened on a clock two hours on.
///
/// The claim was taken under the suite's one-hour lease at `AT_NOW`, so the same directory read
/// at `AT_LATER` is the same holder's name against a claim that stopped excluding an hour ago --
/// which is the only shape in which a lapse can be observed at all.
pub(crate) fn Board_After_The_Lease_Lapsed(name: &str) -> BoardOnDisk
{
    let BoardOnDisk { directory, mut ledger } = Board_At(name, vec![Item("T-1", &["a.rs"])]);
    Take(&mut ledger, "T-1", &Holder::from("agent-a"));
    drop(ledger);

    let ledger = Ledger_At(&directory, &AT_LATER);
    return BoardOnDisk { directory, ledger };
}

/// One agent claims one item for the standard lease, and it is expected to succeed.
pub(crate) fn Take<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &Holder<'_>)
{
    ledger
        .Claim(&ItemId::New(item), holder.As_Text(), LEASE)
        .unwrap_or_else(|refusal| panic!("the fixture claim was refused: {}", refusal.Describe()));
}

pub(crate) fn Paths(paths: &[&str]) -> Vec<String>
{
    return paths.iter().map(|path| return (*path).to_owned()).collect();
}

/// The one item on a board, read back off disk.
pub(crate) fn Only_Item<Files, TimeSource, Lock>(ledger: &FileLedger<Files, TimeSource, Lock>) -> LedgerItem
where
    Files: nomos_platform::FileSystem,
    TimeSource: Clock,
    Lock: FilesystemLock,
{
    return ledger
        .Load()
        .expect("the board is readable")
        .items
        .into_iter()
        .next()
        .expect("the board has an item");
}

/// One item off a board, by id, read back off disk.
pub(crate) fn Named<Files, TimeSource, Lock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    id: &str,
) -> LedgerItem
where
    Files: nomos_platform::FileSystem,
    TimeSource: Clock,
    Lock: FilesystemLock,
{
    return ledger
        .Load()
        .expect("the board is readable")
        .items
        .into_iter()
        .find(|item| return item.id == ItemId::New(id))
        .expect("the item is on the board");
}
