//! A held territory can be enlarged by its holder, and every way of getting it wrong is refused.
//!
//! A reservation is authored before the change it reserves has been attempted, so it is a
//! prediction. Until `OD-LEDGER-039` the ledger's only answer to a prediction that was short by
//! one file was to end the item: abandon, decline, re-author, and re-author every dependent the
//! decline stranded. Worse, the board then carried no record that the prediction had been
//! short, because a decline stores a holder, a timestamp and free prose.
//!
//! Every test here has a negative control, for the reason `exclusion_holds.rs` gives: a guard
//! nobody has watched fail is a test that would pass just as happily if the thing it checks
//! were deleted. The two that matter most are the contention pair and the lapse pair, because
//! each has a sibling in which the *same* widening is granted — so neither can be satisfied by
//! an implementation that simply refuses everything.

use nomos_ledger::{
    ClaimRefusal, ExclusionLedger, FileLedger, Holder, ItemId, ItemKind, ItemOrigin, ItemState,
    LedgerDocument, LedgerItem, SCHEMA_VERSION, Territory,
};
use nomos_platform::{
    Clock, CrossProcessLock, DeterminismStrength, LockAcquisition, LockError, ReproducibilityScope,
    Strategy, Timestamp, TraceEquivalence,
};
use nomos_platform_std::{FileLock, FileLockGuard, StdFileSystem};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const NOW: i64 = 1_000_000;
const LEASE: Duration = Duration::from_secs(3_600);

/// Held still, so that "when it was widened" is arithmetic rather than a sleep.
struct FixedClock(i64);

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

static AT_NOW: FixedClock = FixedClock(NOW);

/// Two hours on, by which time the one-hour lease every test here takes has lapsed.
static AT_LATER: FixedClock = FixedClock(NOW + 7_200);

fn Item(id: &str, files: &[&str]) -> LedgerItem
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

fn Temporary_Directory(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-widen-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    return path;
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

/// A ledger on a fresh temporary directory, already holding the board it starts from.
fn Board_At(
    name: &str,
    items: Vec<LedgerItem>,
) -> (PathBuf, FileLedger<StdFileSystem, &'static FixedClock, FileLock>)
{
    let directory = Temporary_Directory(name);
    let ledger = Ledger_At(&directory, &AT_NOW);
    ledger
        .Save(&LedgerDocument { schema_version: SCHEMA_VERSION, items })
        .expect("a fresh ledger is valid");

    return (directory, ledger);
}

fn Take<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &str)
{
    ledger
        .Claim(&ItemId::New(item), holder, LEASE)
        .unwrap_or_else(|refusal| panic!("the fixture claim was refused: {}", refusal.Describe()));
}

fn Paths(paths: &[&str]) -> Vec<String>
{
    return paths.iter().map(|path| return (*path).to_owned()).collect();
}

/// The one item on a board, read back off disk.
fn Only_Item<Files, TimeSource, Lock>(ledger: &FileLedger<Files, TimeSource, Lock>) -> LedgerItem
where
    Files: nomos_platform::FileSystem,
    TimeSource: Clock,
    Lock: CrossProcessLock,
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
fn Named<Files, TimeSource, Lock>(
    ledger: &FileLedger<Files, TimeSource, Lock>,
    id: &str,
) -> LedgerItem
where
    Files: nomos_platform::FileSystem,
    TimeSource: Clock,
    Lock: CrossProcessLock,
{
    return ledger
        .Load()
        .expect("the board is readable")
        .items
        .into_iter()
        .find(|item| return item.id == ItemId::New(id))
        .expect("the item is on the board");
}


// ---------------------------------------------------------------------------------------
// What a widening does when it is allowed
// ---------------------------------------------------------------------------------------

/// The whole of the happy path: the territory grows and the growth is kept.
///
/// Both halves, because either alone is satisfiable by an implementation with the defect the
/// other catches. A territory that grew without a record is the measurement lost, which is the
/// thing `OD-LEDGER-039` is mostly about; a record without the growth is a note about work the
/// holder still cannot do.
#[test]
fn Test_A_Holder_Should_Widen_Their_Own_Territory_And_The_Widening_Should_Be_Kept()
{
    let (_directory, mut ledger) = Board_At("kept", vec![Item("T-1", &["a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let added = ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["b.rs", "c.rs"]))
        .expect("the holder may widen their own item");

    assert_eq!(added, Paths(&["b.rs", "c.rs"]), "the verb reports what it added");

    let item = Only_Item(&ledger);
    assert_eq!(
        item.territory.paths,
        Paths(&["a.rs", "b.rs", "c.rs"]),
        "the reserved paths must be the original ones plus the added ones, in that order"
    );
    assert_eq!(item.widened.len(), 1, "one widening happened, so one is recorded");
    let recorded = item.widened.first().expect("the length was just asserted");
    assert_eq!(recorded.holder, "agent-a", "who found the reservation short");
    assert_eq!(recorded.added, Paths(&["b.rs", "c.rs"]), "and exactly what they added");
    assert_eq!(
        recorded.widened_at,
        Timestamp::From_Unix_Seconds(NOW),
        "and when, so an escape rate can be measured over time rather than merely counted"
    );
}

/// Two widenings are two rows, kept oldest first.
///
/// The falsifier for coalescing. An implementation that merged them into one row with four
/// paths would leave the territory correct and every other test here green, while destroying
/// the fact that this holder was wrong twice — which is the difference between one bad
/// prediction and a reservation that was never a serious attempt.
#[test]
fn Test_Two_Widenings_Should_Be_Two_Rows_Rather_Than_One_Coalesced_One()
{
    let (_directory, mut ledger) = Board_At("multiplicity", vec![Item("T-1", &["a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["b.rs"]))
        .expect("the first widening is granted");
    ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["c.rs", "d.rs"]))
        .expect("the second widening is granted");

    let item = Only_Item(&ledger);
    assert_eq!(item.widened.len(), 2, "two widenings, two rows");
    let [first, second] = item.widened.as_slice()
    else
    {
        panic!("the length was just asserted");
    };
    assert_eq!(first.added, Paths(&["b.rs"]), "oldest first");
    assert_eq!(second.added, Paths(&["c.rs", "d.rs"]), "and the later one after it");
    assert_eq!(
        item.territory.paths,
        Paths(&["a.rs", "b.rs", "c.rs", "d.rs"]),
        "and the territory carries every path either of them added"
    );
}

/// A path already reserved adds nothing, and nothing is recorded.
///
/// Recording it would overstate the escape, and the escape rate is the one number these rows
/// exist to carry honestly. Two spellings of one path are one path, for the same reason and by
/// the same normalization `Territory::Intersect` uses: a holder who re-reserves `./a.rs` has
/// not widened anything.
#[test]
fn Test_A_Path_Already_Reserved_Should_Add_Nothing_And_Record_Nothing()
{
    let (_directory, mut ledger) = Board_At("already", vec![Item("T-1", &["a.rs", "dir/b.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let added = ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["a.rs", "./dir/b.rs"]))
        .expect("asking for what you already hold is not an error");

    assert!(added.is_empty(), "nothing was added, so nothing is reported as added: {added:?}");

    let item = Only_Item(&ledger);
    assert_eq!(item.territory.paths, Paths(&["a.rs", "dir/b.rs"]), "the territory is untouched");
    assert!(
        item.widened.is_empty(),
        "a widening that added nothing must not be recorded as one: {:?}",
        item.widened
    );
}

/// The same path named twice in one widening is one path.
#[test]
fn Test_A_Path_Named_Twice_In_One_Widening_Should_Be_Added_Once()
{
    let (_directory, mut ledger) = Board_At("twice", vec![Item("T-1", &["a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let added = ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["b.rs", "b.rs"]))
        .expect("the holder may widen their own item");

    assert_eq!(added, Paths(&["b.rs"]), "one path was added, however many times it was named");
    assert_eq!(Only_Item(&ledger).territory.paths, Paths(&["a.rs", "b.rs"]));
}

/// Widening only ever adds, so every path reserved before one is still reserved after it.
///
/// There is no argument on the verb that could express a replacement territory, which is what
/// makes this a property of the signature rather than of the body. The assertion is here so
/// that a later change adding one fails a test rather than merely a review: dropping a path
/// drops the `done_when` clause that path carried.
#[test]
fn Test_Widening_Should_Never_Remove_A_Path()
{
    let (_directory, mut ledger) = Board_At("superset", vec![Item("T-1", &["a.rs", "b.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let before = Only_Item(&ledger).territory.paths;
    ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["c.rs"]))
        .expect("the holder may widen their own item");
    let after = Only_Item(&ledger).territory.paths;

    for path in &before
    {
        assert!(after.contains(path), "{path} was reserved before the widening and is not after");
    }
}


// ---------------------------------------------------------------------------------------
// What refuses a widening
// ---------------------------------------------------------------------------------------

/// Ground a peer actively holds refuses the widening, and the item is left alone.
///
/// This is the refusal the whole verb has to earn. A holder who could reach any path by
/// widening would have the exclusion the board is for, and the check would never be asked.
#[test]
fn Test_A_Widening_Onto_Ground_A_Peer_Holds_Should_Be_Refused()
{
    let (_directory, mut ledger) = Board_At(
        "contested",
        vec![Item("T-1", &["a.rs"]), Item("T-2", &["b.rs"])],
    );
    Take(&mut ledger, "T-1", "agent-a");
    Take(&mut ledger, "T-2", "agent-b");

    let refusal = ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["b.rs"]))
        .expect_err("b.rs is held by agent-b through T-2");

    assert!(
        matches!(refusal, ClaimRefusal::HeldBy { .. }),
        "the refusal must name the live claim in the way a claim's would: {}",
        refusal.Describe()
    );

    let item = Named(&ledger, "T-1");
    assert_eq!(item.territory.paths, Paths(&["a.rs"]), "a refused widening writes no path");
    assert!(item.widened.is_empty(), "and records no widening: {:?}", item.widened);
}

/// The negative control for the test above: the same widening, granted once nothing holds the
/// ground.
///
/// Without this, an implementation that refused every widening would satisfy the contention
/// test — and the point of the verb is that it grants the ones it should.
#[test]
fn Test_The_Same_Widening_Should_Be_Granted_When_No_Peer_Holds_The_Ground()
{
    let (_directory, mut ledger) = Board_At(
        "uncontested",
        vec![Item("T-1", &["a.rs"]), Item("T-2", &["b.rs"])],
    );
    Take(&mut ledger, "T-1", "agent-a");

    let added = ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["b.rs"]))
        .expect("T-2 reserves b.rs but nobody is holding T-2");

    assert_eq!(added, Paths(&["b.rs"]), "an unclaimed peer's territory excludes nobody");
}

/// Somebody who is not the holder may not widen, even onto free ground.
#[test]
fn Test_A_Widening_By_Somebody_Who_Is_Not_The_Holder_Should_Be_Refused()
{
    let (_directory, mut ledger) = Board_At("notholder", vec![Item("T-1", &["a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let refusal = ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-b"), &Paths(&["b.rs"]))
        .expect_err("agent-b does not hold T-1");

    assert!(
        matches!(refusal, ClaimRefusal::StillHeld { .. }),
        "the refusal must say who does hold it: {}",
        refusal.Describe()
    );
    assert_eq!(Only_Item(&ledger).territory.paths, Paths(&["a.rs"]), "and write nothing");
}

/// An item nobody holds may not be widened.
///
/// Territory on a `Ready` item is the author's prediction and has not been tested against
/// anything yet, so there is no evidence to correct it with. The repair for a reservation that
/// is wrong before any work began is to decline and re-author.
#[test]
fn Test_A_Widening_Of_An_Item_Nobody_Holds_Should_Be_Refused()
{
    let (_directory, mut ledger) = Board_At("unclaimed", vec![Item("T-1", &["a.rs"])]);

    let refusal = ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["b.rs"]))
        .expect_err("nobody holds T-1");

    assert!(
        matches!(refusal, ClaimRefusal::NotClaimable { .. }),
        "an unheld item has no holder to be: {}",
        refusal.Describe()
    );
    assert!(Only_Item(&ledger).widened.is_empty(), "and nothing is written");
}

/// A holder whose lease has run out may not widen, and their name still matching is exactly
/// why this needs its own refusal.
///
/// A lapsed claim stops excluding — `store/refusal.rs` records that for takeovers — so another
/// item may since have been claimed over the very files this one reserves. Enlarging a
/// reservation that currently excludes nobody is that hazard reached through a new verb. The
/// remedy named is `takeover`, which is the verb that makes the claim live again.
#[test]
fn Test_A_Widening_By_A_Holder_Whose_Lease_Has_Run_Out_Should_Be_Refused()
{
    let directory = Temporary_Directory("lapsed");
    {
        let mut ledger = Ledger_At(&directory, &AT_NOW);
        ledger
            .Save(&LedgerDocument {
                schema_version: SCHEMA_VERSION,
                items: vec![Item("T-1", &["a.rs"])],
            })
            .expect("a fresh ledger is valid");
        Take(&mut ledger, "T-1", "agent-a");
    }

    let mut later = Ledger_At(&directory, &AT_LATER);
    let refusal = later
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["b.rs"]))
        .expect_err("agent-a's lease ran out an hour ago");

    assert!(
        matches!(refusal, ClaimRefusal::Lapsed { .. }),
        "a dead lease is not authorization, however well the name matches: {}",
        refusal.Describe()
    );
    assert!(
        refusal.Describe().contains("takeover"),
        "and the remedy must be named: {}",
        refusal.Describe()
    );

    let item = Only_Item(&later);
    assert_eq!(item.territory.paths, Paths(&["a.rs"]), "a refused widening writes no path");
    assert!(item.widened.is_empty(), "and records no widening");
}

/// The negative control for the lapse: the same holder, the same item, before the lease runs
/// out.
#[test]
fn Test_The_Same_Holder_Should_Widen_While_The_Lease_Is_Still_Live()
{
    let (_directory, mut ledger) = Board_At("live", vec![Item("T-1", &["a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let added = ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["b.rs"]))
        .expect("the lease has not run out");

    assert_eq!(added, Paths(&["b.rs"]), "a live lease is what the refusal above turns on");
}


// ---------------------------------------------------------------------------------------
// Where the decision is made
// ---------------------------------------------------------------------------------------

/// What a lock acquisition saw, and what the file held when it was released.
#[derive(Default)]
struct LockLog
{
    acquisitions: u32,
    at_release: Vec<String>,
}

/// A real lock that records when it was taken and what the document held as it was released.
///
/// The falsifier for atomicity, which sharing the exclusion check does not buy. Two widenings
/// running at once can each read a board on which their own added paths are free and jointly
/// write the overlap the check exists to prevent, and the only thing that stops it is deciding
/// and writing without releasing the lock in between. `OD-LEDGER-015` is what the three verbs
/// before this one cost by deciding outside it.
///
/// A counting fake alone would only catch an implementation that took no lock. Reading the
/// document *inside* `Drop`, while the inner guard is still held, is what catches the worse
/// shape: a decision made under the lock and a write performed after releasing it. The field
/// order matters and is the mechanism — `Drop::drop` runs before the struct's fields are
/// dropped, so the real lock is still held at the moment the snapshot is taken.
struct WatchingLock
{
    inner: FileLock,
    document: PathBuf,
    log: Arc<Mutex<LockLog>>,
}

impl Strategy for WatchingLock
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::None;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::NotApplicable;
}

struct WatchingGuard
{
    document: PathBuf,
    log: Arc<Mutex<LockLog>>,
    /// The real lock, released when this guard finishes dropping.
    ///
    /// Last, and that is the mechanism rather than a style choice: `Drop::drop` runs before a
    /// struct's fields are dropped, so the lock is still held while the snapshot below is
    /// taken. Read by the snapshot through its own existence rather than by name, which is
    /// what the drop below says out loud so that nothing prunes it as unused.
    held: FileLockGuard,
}

impl Drop for WatchingGuard
{
    fn drop(&mut self)
    {
        // The whole point of the snapshot: what the document held while the lock was still
        // ours. Naming `held` here is deliberate -- it is the thing making that true, and a
        // field nothing mentions is a field somebody deletes.
        let held: &FileLockGuard = &self.held;
        let _still_locked = std::ptr::from_ref(held);

        let text = std::fs::read_to_string(&self.document).unwrap_or_default();
        self.log.lock().expect("the log is not poisoned").at_release.push(text);
    }
}

impl CrossProcessLock for WatchingLock
{
    type Guard = WatchingGuard;

    fn Acquire(
        &self,
        holder: &str,
        wait_limit: Duration,
        stale_after: Duration,
    ) -> Result<LockAcquisition<Self::Guard>, LockError>
    {
        let acquired = self.inner.Acquire(holder, wait_limit, stale_after)?;
        let mut log = self.log.lock().expect("the log is not poisoned");
        log.acquisitions = log.acquisitions.saturating_add(1);
        drop(log);

        return Ok(LockAcquisition {
            guard: WatchingGuard {
                document: self.document.clone(),
                log: Arc::clone(&self.log),
                held: acquired.guard,
            },
            broke_stale: acquired.broke_stale,
        });
    }
}

#[test]
fn Test_A_Widening_Should_Decide_And_Write_Inside_One_Lock_Acquisition()
{
    let directory = Temporary_Directory("atomic");
    let document = directory.join("ledger.json");
    let log = Arc::new(Mutex::new(LockLog::default()));

    let mut ledger = FileLedger::At(
        document.clone(),
        StdFileSystem,
        &AT_NOW,
        WatchingLock {
            inner: FileLock::At(directory.join("ledger.lock")),
            document: document.clone(),
            log: Arc::clone(&log),
        },
    );
    ledger
        .Save(&LedgerDocument { schema_version: SCHEMA_VERSION, items: vec![Item("T-1", &["a.rs"])] })
        .expect("a fresh ledger is valid");
    Take(&mut ledger, "T-1", "agent-a");

    log.lock().expect("the log is not poisoned").at_release.clear();
    let taken_before = log.lock().expect("the log is not poisoned").acquisitions;

    ledger
        .Widen(&ItemId::New("T-1"), Holder::from("agent-a"), &Paths(&["b.rs"]))
        .expect("the holder may widen their own item");

    let log = log.lock().expect("the log is not poisoned");
    assert_eq!(
        log.acquisitions - taken_before,
        1,
        "one widening takes the lock exactly once: twice would mean the decision and the write \
         each took their own, with the board free to move between them"
    );
    let [at_release] = log.at_release.as_slice()
    else
    {
        panic!("one widening releases the lock exactly once, and this released {} times", log.at_release.len());
    };
    assert!(
        at_release.contains("b.rs"),
        "the widening must already be on disk when the lock is released, or it was written \
         after the lock was given up and another writer could have been between the two"
    );
}


// ---------------------------------------------------------------------------------------
// What the document promises about itself
// ---------------------------------------------------------------------------------------

/// A row written before this field existed is refused rather than read as never widened.
///
/// The direction `#[serde(default)]` decides, and the reason this field carries none. An item
/// that predates the verb may well have been widened by the only means there was — declining it
/// and re-authoring it with more paths — so defaulting the field to empty would be a claim
/// about history the document cannot support. The board is migrated instead.
///
/// The schema number is deliberately not what performs this refusal, and `OD-LEDGER-008` is
/// why. It is consulted after a parse has already failed, and only to choose which sentence
/// the operator reads.
#[test]
fn Test_An_Item_Written_Without_The_Widened_Field_Should_Be_Refused()
{
    let directory = Temporary_Directory("unmigrated");
    let ledger = Ledger_At(&directory, &AT_NOW);

    ledger
        .Save(&LedgerDocument { schema_version: SCHEMA_VERSION, items: vec![Item("T-1", &["a.rs"])] })
        .expect("a fresh ledger is valid");

    let path = directory.join("ledger.json");
    let written = std::fs::read_to_string(&path).expect("the board was written");
    assert!(
        written.contains("\"widened\""),
        "the field must be serialized on every item, whatever it holds -- counting the key \
         across the file is how a stale writer is detected, and a key whose presence depends \
         on its content cannot be counted"
    );

    // Removed as JSON rather than as text, so the fixture is a document missing one key
    // rather than one that does not parse at all -- which would pass this test for the wrong
    // reason, and would pass it just as happily if the field carried a default.
    let mut document: serde_json::Value =
        serde_json::from_str(&written).expect("the board this build wrote is JSON");
    let removed = document
        .get_mut("items")
        .and_then(|items| return items.get_mut(0))
        .and_then(serde_json::Value::as_object_mut)
        .expect("the board this build wrote has one item, and an item is an object")
        .remove("widened");
    assert!(removed.is_some(), "the fixture must actually remove the field");
    let unmigrated = serde_json::to_string_pretty(&document).expect("the fixture serializes");
    serde_json::from_str::<serde_json::Value>(&unmigrated)
        .expect("the fixture must still be well-formed JSON, or this tests the parser");
    std::fs::write(&path, &unmigrated).expect("the fixture is written");

    let error = ledger.Load().expect_err("a row missing the field is not a row with an empty one");
    let said = format!("{error:?}");
    assert!(
        said.contains("widened"),
        "the refusal must name the field an operator has to migrate: {said}"
    );
}
