//! The board every test here reads, the scratch copy it claims against, and the two
//! path questions the ledger's own rule answers.

use nomos_ledger::{
    ClaimRefusal, ExclusionLedger, FileLedger, ItemId, ItemState, LedgerDocument, LedgerItem,
    Normalize_Path, Territory,
};
use nomos_platform::{Clock, Timestamp};
use nomos_platform_std::{FileLock, StdFileSystem};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The directory every record in this repository lives in, normalized.
pub(crate) const RECORD_DIRECTORY: &str = "docs/records";

pub(crate) struct FixedClock(i64);

impl Clock for &FixedClock
{
    fn Now(&self) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(self.0);
    }
}

const NOW: i64 = 1_000_000;

/// The repository root, from this crate's manifest directory.
pub(crate) fn Repository_Root() -> PathBuf
{
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/substrate/nomos-ledger -> the repository.
    for _ in 0..3
    {
        root.pop();
    }
    return root;
}

/// The ledger this repository actually runs on.
pub(crate) fn Real_Ledger() -> LedgerDocument
{
    let path = Repository_Root().join("work").join("ledger.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("the repository ledger must be readable: {error}"));
    return serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("the repository ledger must parse: {error}"));
}

/// Whether an item is still competing for territory.
///
/// `Done` and `Declined` items hold no claim and exclude nobody, so their territories are
/// history rather than reservations and are deliberately left as they were authored.
pub(crate) fn Is_Open(item: &LedgerItem) -> bool
{
    return matches!(item.state, ItemState::Ready | ItemState::Claimed);
}

/// The record paths an item reserves.
pub(crate) fn Reserved_Records(item: &LedgerItem) -> Vec<String>
{
    return item
        .territory
        .paths
        .iter()
        .map(|path| Normalize_Path(path))
        .filter(|path| path.starts_with(RECORD_DIRECTORY))
        .collect();
}

/// A temporary board that removes itself when the test holding it ends.
///
/// Every test here used to close with its own `remove_dir_all`, which is a line that only
/// runs when the test passes: a failed assertion unwinds straight past it. `Drop` runs on the
/// unwind too, so the tree is cleared exactly when the value goes out of scope.
pub(crate) struct Scratch(PathBuf);

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

fn Temp_Dir(name: &str) -> Scratch
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-record-lock-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    return Scratch(path);
}

/// The ledger every test here opens: a temporary file, read against the shared clock.
pub(crate) type Board = FileLedger<StdFileSystem, &'static FixedClock, FileLock>;

/// A doctored board on disk, and the ledger open over it.
pub(crate) fn Saved(name: &str, document: &LedgerDocument) -> (Scratch, Board)
{
    let directory = Temp_Dir(name);
    let ledger = Ledger_At(&directory, &AT_NOW);

    ledger.Save(document).expect("the doctored board is still a valid ledger");

    return (directory, ledger);
}

/// Claims an item, or fails naming what the refusal means for the property under test.
pub(crate) fn Claimed(ledger: &mut Board, writer: &ItemId, agent: &str, blame: &str)
{
    ledger
        .Claim(writer, agent, Duration::from_secs(3_600))
        .unwrap_or_else(|refusal| panic!("{blame}: {}", refusal.Describe()));
}

/// The clock every test that does not move time shares.
///
/// A `'static` clock rather than a local one in each test: nothing here moves time, so one
/// shared reading is what every ledger these tests open is read against.
static AT_NOW: FixedClock = FixedClock(NOW);

/// A board holding two open record writers, and their identifiers.
///
/// Every contest below needs exactly this: a copy nobody holds, and two writers to set
/// against each other. The pair is *constructed* rather than taken from the live board,
/// which `OD-LEDGER-032` decided and which costs nothing here: both callers overwrite both
/// territories before contesting anything, so neither ever read what the real items
/// reserved. Borrowing them made the contest possible only while somebody happened to be
/// mid-work, and impossible on a board at rest.
///
/// This is not the same choice as `OD-LEDGER-004`'s. That record reads the *real* board
/// where the real board is the subject — what the items actually say — and this is not one
/// of those places.
pub(crate) fn Two_Record_Writers() -> (LedgerDocument, ItemId, ItemId)
{
    let document = Constructed_Writers(2);
    let writers = Writer_Ids(&document);

    let (Some(first), Some(second)) = (writers.first().cloned(), writers.get(1).cloned())
    else
    {
        panic!("a board this function built with two writers must hold two")
    };

    return (document, first, second);
}
/// A board carrying exactly `count` open record writers, constructed rather than borrowed.
///
/// `OD-LEDGER-032` is why this exists. Every control in this file used to take its subject
/// from `Unclaimed_Copy`, which meant a control could only prove the search had teeth on a
/// board that happened to be busy. On a board at rest — the normal end state of finished
/// work, and what CI checks out — the control found nothing to widen, reported nothing, and
/// failed for a reason no commit contained.
///
/// Built from real items rather than from literals, so the shape stays whatever
/// `LedgerItem` is today: a field added to that struct cannot leave this constructor
/// compiling against a shape the real board no longer has. Every item is reopened as
/// `Ready`, stripped of its claim and its dependencies, and pointed at a record of its own
/// under a `docs/records/OD-CONSTRUCTED-` identifier that no real record uses.
///
/// # Panics
///
/// If the real ledger carries fewer than `count` items at all. It carries hundreds, and a
/// board that did not could not have produced the defect this file measures.
pub(crate) fn Constructed_Writers(count: usize) -> LedgerDocument
{
    let mut document = Real_Ledger();

    assert!(
        document.items.len() >= count,
        "the real ledger must carry at least {count} items to build a subject from; got {}",
        document.items.len()
    );
    document.items.truncate(count);

    for (ordinal, item) in document.items.iter_mut().enumerate()
    {
        item.state = ItemState::Ready;
        item.claim = None;
        item.depends_on.clear();
        item.blocked = None;
        item.territory =
            Territory::Of_Files([format!("{RECORD_DIRECTORY}/OD-CONSTRUCTED-{ordinal:03}")]);
    }

    return document;
}

/// Puts a doctored board on disk and lets two agents contest it, first come first served.
///
/// The refusal handed back is the second agent's, which is what every test using this is
/// about; the directory comes back so that it outlives the assertion rather than being
/// cleared while the ledger is still open over it.
pub(crate) fn Contested(
    name: &str,
    document: &LedgerDocument,
    first: &ItemId,
    second: &ItemId,
) -> (Scratch, ClaimRefusal)
{
    let (directory, mut ledger) = Saved(name, document);

    ledger
        .Claim(first, "agent-a", Duration::from_secs(3_600))
        .expect("the first claim is uncontended");
    let refusal = ledger
        .Claim(second, "agent-b", Duration::from_secs(3_600))
        .expect_err("the second of a contesting pair must be refused");

    return (directory, refusal);
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

/// The real ledger with every live claim cleared.
///
/// Whoever happens to be holding an item when the suite runs must not change what it
/// concludes, and the question here is whether two items *could* be held at once — not
/// whether they are free this second.
pub(crate) fn Unclaimed_Copy() -> LedgerDocument
{
    let mut document = Real_Ledger();
    for item in &mut document.items
    {
        if Is_Open(item)
        {
            item.state = ItemState::Ready;
            item.claim = None;
        }
    }
    return document;
}

/// Every open item that will write a record and could be claimed on its own merits.
///
/// An unmet dependency refuses a claim for reasons that have nothing to do with territory
/// and would make every assertion here fail for the wrong reason.
pub(crate) fn Record_Writers(document: &LedgerDocument) -> Vec<&LedgerItem>
{
    return document
        .items
        .iter()
        .filter(|item| Is_Open(item))
        .filter(|item| !Reserved_Records(item).is_empty())
        .filter(|item| item.depends_on.is_empty())
        .collect();
}

/// An item's territory with everything but its records removed.
///
/// The projection the acceptance property is stated over. `P10-RECORD-LOCK` bought one
/// thing and one thing only: that the *records* two items reserve stop excluding them from
/// each other. Whether they also share code is a separate question with a separate answer,
/// and asserting the two together is what made the guarantee depend on who was on the
/// board — see `OD-LEDGER-007`.
pub(crate) fn Only_Records(item: &LedgerItem) -> Territory
{
    return Territory::Of_Files(Reserved_Records(item));
}

/// The identifiers of every open record writer on a board.
pub(crate) fn Writer_Ids(document: &LedgerDocument) -> Vec<ItemId>
{
    return Record_Writers(document)
        .iter()
        .map(|item| return item.id.clone())
        .collect();
}

/// Reduces every named item to the records it reserves, and nothing else.
pub(crate) fn Project_Onto_Records(document: &mut LedgerDocument, writers: &[ItemId])
{
    for item in &mut document.items
    {
        if writers.contains(&item.id)
        {
            item.territory = Only_Records(item);
        }
    }
}

/// Whether two paths exclude each other, decided by the ledger's own rule.
///
/// Single-path territories rather than a containment check written here. `a/b` contains
/// `a/b/c` and two spellings of one path are one path, and a second implementation of
/// either would be a second answer waiting to disagree with `Territory::Intersect`.
pub(crate) fn Paths_Collide(left: &str, right: &str) -> bool
{
    return !Territory::Of_Files([left])
        .Intersect(&Territory::Of_Files([right]))
        .Permits_Concurrency();
}

/// Whether reserving `reserved` reserves `declared` — the coarse direction, not either one.
///
/// [`Paths_Collide`] answers a symmetric question, because exclusion is symmetric: an item
/// reserving a directory and an item reserving a file inside it exclude each other, and
/// which of them is broader does not change that. Whether a *declared serializer* is still
/// serializing is not that question. It asks whether anybody still reserves the coarse path
/// itself, and under the symmetric reading `tests/contract/surface/nomos-ledger.txt` counts
/// as reserving `tests/contract` — so narrowing a reservation never reduces the count and
/// only deleting the item does. The register could not empty, which is the one thing
/// `OD-LEDGER-007` says it is for. Found while closing `P10-SURFACE-GRAIN`; `OD-LEDGER-011`
/// records it.
///
/// The direction is taken from the collision rather than decided again beside it. If two
/// paths collide under this ledger's rule then one contains the other, and the shorter
/// normalized spelling is the container — the same tie-break [`super::serializers::Shared_Paths`]
/// already uses to name the broader of two paths. A containment check written out here would
/// be a second answer waiting to disagree with `Territory::Intersect`.
pub(crate) fn Covers(reserved: &str, declared: &str) -> bool
{
    return Paths_Collide(reserved, declared)
        && Normalize_Path(reserved).len() <= Normalize_Path(declared).len();
}
