//! The board every test here reads, the scratch copy it claims against, and the two
//! path questions the ledger's own rule answers.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_ledger::{
    ClaimRefusal, ExclusionLedger, FileLedger, Holder, ItemId, ItemState, LedgerDocument,
    LedgerItem, Normalize_Path, Territory,
};
use nomos_platform::{Clock, Timestamp};
use nomos_platform_std::{FileLock, StdFileSystem};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The directory every record in this repository lives in, normalized.
pub(crate) const RECORD_DIRECTORY: &str = "docs/records";

pub(crate) struct FixedClock(i64);

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

const NOW: i64 = 1_000_000;

/// The lease every claim in this suite takes. Long enough that no test here can lapse one
/// by accident; nothing in this file moves time.
const LEASE: Duration = Duration::from_secs(3_600);

/// How many directory levels this crate's manifest directory sits below the repository root.
///
/// `crates/substrate/nomos-ledger`, so three: the crate, then `substrate`, then `crates`.
const LEVELS_BELOW_THE_REPOSITORY_ROOT: usize = 3;

/// How many record writers a contest needs: one to hold, one to be refused by it.
///
/// Every constructed subject in this suite is built with exactly this many, and the
/// controls that prove the searches have teeth are all contests between the two.
pub(crate) const CONTESTING_WRITERS: usize = 2;

/// The repository root, from this crate's manifest directory.
pub(crate) fn Repository_Root() -> PathBuf
{
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..LEVELS_BELOW_THE_REPOSITORY_ROOT
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
        if let Err(error) = std::fs::remove_dir_all(&self.0)
        {
            // A `Drop` impl runs on the unwinding path too, so this must stay infallible:
            // a panic here aborts the process and buries the assertion already failing.
            eprintln!(
                "could not clear the scratch directory {}: {error}",
                self.0.display()
            );
        }
    }
}

impl Scratch
{
    pub(crate) fn As_Path(&self) -> &Path
    {
        return &self.0;
    }
}

fn Temporary_Directory(name: &str) -> Scratch
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-record-lock-{name}-{}", std::process::id()));

    // A process id that repeats finds the previous run's tree still here. Nothing to clear
    // is the ordinary case and is not worth a word; anything else means a stale tree is
    // about to be read as this run's own.
    match std::fs::remove_dir_all(&path)
    {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => eprintln!(
            "a leftover scratch directory at {} could not be cleared: {error}",
            path.display()
        ),
    }

    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    return Scratch(path);
}

/// The ledger every test here opens: a temporary file, read against the shared clock.
pub(crate) type Board = FileLedger<StdFileSystem, &'static FixedClock, FileLock>;

/// A doctored board on disk, and the ledger open over it.
///
/// Named fields rather than a pair, because the two are not interchangeable: the scratch
/// directory has to outlive the ledger that is open over it, and a caller that read them by
/// position would have to remember which was which to keep that true.
pub(crate) struct SavedBoard
{
    pub(crate) scratch: Scratch,
    pub(crate) ledger: Board,
}

/// Writes a doctored board into a fresh scratch directory and opens a ledger over it.
pub(crate) fn Board_Saved_In_Scratch(name: &str, document: &LedgerDocument) -> SavedBoard
{
    let directory = Temporary_Directory(name);
    let ledger = Ledger_At(directory.As_Path(), &AT_NOW);

    // Every subject in this suite is an item whose territory has been replaced by a
    // projection of it -- the records it reserves, the record directory, a contested path.
    // A recorded widening names paths of the *real* territory, so carrying one onto a
    // projection produces an item claiming to have added ground it does not reserve, which
    // `Validate_Document` correctly refuses. The projection is not that item and does not
    // inherit its history; dropping it here rather than at seven call sites is what keeps
    // the next projection from having to remember.
    let mut document = document.clone();
    for item in &mut document.items
    {
        item.widened.clear();
    }

    ledger.Save(&document).expect("the doctored board is still a valid ledger");

    return SavedBoard {
        scratch: directory,
        ledger,
    };
}

/// Claims an item, or fails naming what the refusal means for the property under test.
///
/// The agent is a [`Holder`] rather than a `&str` so the two cannot be swapped at a call
/// site: one is who is claiming and the other is the prose a refusal is reported with, and
/// a bare `&str` in both positions says only that both are text.
pub(crate) fn Claim_Item_For_Writer(ledger: &mut Board, writer: &ItemId, agent: Holder<'_>, blame: &str)
{
    ledger
        .Claim(writer, agent.As_Text(), LEASE)
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
pub(crate) struct TwoWriters
{
    pub(crate) document: LedgerDocument,
    pub(crate) first: ItemId,
    pub(crate) second: ItemId,
}

pub(crate) fn Two_Record_Writers() -> TwoWriters
{
    let document = Constructed_Writers(CONTESTING_WRITERS);
    let writers = Writer_Ids(&document);

    let (Some(first), Some(second)) = (writers.first().cloned(), writers.get(1).cloned())
    else
    {
        panic!("a board this function built with two writers must hold two")
    };

    return TwoWriters {
        document,
        first,
        second,
    };
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

/// The refusal a contest produced, and the scratch directory it has to outlive.
///
/// Named fields rather than a pair: the refusal is the second agent's, which is what every
/// test using this is about, and the directory must not be cleared while the ledger that
/// produced it is still open over it.
pub(crate) struct Contest
{
    pub(crate) scratch: Scratch,
    pub(crate) refusal: ClaimRefusal,
}

/// Puts a doctored board on disk and lets two agents contest it, first come first served.
pub(crate) fn Board_Contested_By_Two_Agents(
    name: &str,
    document: &LedgerDocument,
    first: &ItemId,
    second: &ItemId,
) -> Contest
{
    let SavedBoard {
        scratch,
        mut ledger,
    } = Board_Saved_In_Scratch(name, document);

    ledger
        .Claim(first, "agent-a", LEASE)
        .expect("the first claim is uncontended");
    let refusal = ledger
        .Claim(second, "agent-b", LEASE)
        .expect_err("the second of a contesting pair must be refused");

    return Contest { scratch, refusal };
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

/// A path as this ledger's own rule reads one.
///
/// Both positions of [`Is_Colliding`] and [`Broader_Of_Two_Paths`] carry this type, and the same one on
/// purpose: exclusion is symmetric, so there is no order at a call site for a reader to get
/// wrong and no wrong answer for a swap to reach. A bare `&str` in both positions would say
/// there was one.
///
/// [`Broader_Of_Two_Paths`]: super::serializers::census::Broader_Of_Two_Paths
pub(crate) struct PathText<'a>(pub(crate) &'a str);

/// Whether two paths exclude each other, decided by the ledger's own rule.
///
/// Single-path territories rather than a containment check written here. `a/b` contains
/// `a/b/c` and two spellings of one path are one path, and a second implementation of
/// either would be a second answer waiting to disagree with `Territory::Intersect`.
pub(crate) fn Is_Colliding(left: PathText<'_>, right: PathText<'_>) -> bool
{
    return !Territory::Of_Files([left.0])
        .Intersect(&Territory::Of_Files([right.0]))
        .Permits_Concurrency();
}

/// Whether reserving `reserved` reserves `declared` — the coarse direction, not either one.
///
/// [`Is_Colliding`] answers a symmetric question, because exclusion is symmetric: an item
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
/// normalized spelling is the container — the same tie-break
/// [`super::serializers::census::Shared_Paths`]
/// already uses to name the broader of two paths. A containment check written out here would
/// be a second answer waiting to disagree with `Territory::Intersect`.
pub(crate) fn Is_Covering(reserved: ReservedPath<'_>, declared: DeclaredPath<'_>) -> bool
{
    return Is_Colliding(PathText(reserved.0), PathText(declared.0))
        && Normalize_Path(reserved.0).len() <= Normalize_Path(declared.0).len();
}

/// A path an item reserves, kept distinct from a path it merely declares so that the two
/// positions of [`Is_Covering`] cannot be swapped at a call site.
///
/// [`Is_Covering`] asks about one direction only — a reservation covers a declaration, never the
/// other way round — and the swap is silent: two `&str` positions accept either order and
/// answer `false` for a question whose whole point is that it is not symmetric.
pub(crate) struct ReservedPath<'a>(pub(crate) &'a str);

/// A path an item declares it will touch, kept distinct from a reserved one for the same
/// reason [`ReservedPath`] exists.
pub(crate) struct DeclaredPath<'a>(pub(crate) &'a str);
