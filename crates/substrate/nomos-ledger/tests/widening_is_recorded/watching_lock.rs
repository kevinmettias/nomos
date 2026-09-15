//! The lock that watches what a widening does between taking it and releasing it.

// ---------------------------------------------------------------------------------------
// Where the decision is made
// ---------------------------------------------------------------------------------------

use crate::fixtures::{
    FixedClock, Holder, Item, LedgerDocument, SCHEMA_VERSION, Take, Temporary_Directory, AT_NOW,
};
use nomos_ledger::FileLedger;
use nomos_platform::{
    CrossProcessLock, DeterminismStrength, LockAcquisition, LockError, ReproducibilityScope,
    Strategy, TraceEquivalence,
};
use nomos_platform_std::{FileLock, FileLockGuard, StdFileSystem};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// What a lock acquisition saw, and what the file held when it was released.
#[derive(Default)]
pub(crate) struct LockLog
{
    pub(crate) acquisitions: u32,
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
/// order matters and is the mechanism -- `Drop::drop` runs before the struct's fields are
/// dropped, so the real lock is still held at the moment the snapshot is taken.
pub(crate) struct WatchingLock
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

pub(crate) struct WatchingGuard
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

        // A poisoned log is read through its own poison rather than unwrapped: this runs
        // during the unwind of a test that is already failing, where a second panic would
        // abort the process instead of reporting anything, and the log's contents are still
        // sound whatever killed the writer.
        let text = std::fs::read_to_string(&self.document).unwrap_or_default();
        let mut log = match self.log.lock()
        {
            Ok(log) => log,
            Err(poisoned) => poisoned.into_inner(),
        };
        log.at_release.push(text);
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

/// The watched board's ledger, named so the test below can take it.
pub(crate) type WatchedBoardLedger = FileLedger<StdFileSystem, &'static FixedClock, WatchingLock>;

/// A board on a fresh temporary directory, written and claimed through a [`WatchingLock`].
pub(crate) struct WatchedBoard
{
    pub(crate) directory: PathBuf,
    pub(crate) log: Arc<Mutex<LockLog>>,
    pub(crate) ledger: WatchedBoardLedger,
}

/// A fresh board whose every lock acquisition is recorded, with the record of the setup
/// cleared so that what the test reads is what its own widening did.
pub(crate) fn Watched_Board_At(name: &str) -> WatchedBoard
{
    let directory = Temporary_Directory(name);
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
    Take(&mut ledger, "T-1", &Holder::from("agent-a"));
    log.lock().expect("the log is not poisoned").at_release.clear();

    return WatchedBoard { directory, log, ledger };
}

/// The widening decided and written inside one acquisition, and already on disk when that
/// acquisition ends.
///
/// Two widenings running at once can each read a board on which their own added paths are
/// free and jointly write the overlap the check exists to prevent, so anything other than one
/// acquisition for the pair is the defect rather than an inefficiency.
pub(crate) fn Assert_One_Lock_Acquisition(log: &Mutex<LockLog>, taken_before: u32)
{
    let log = log.lock().expect("the log is not poisoned");
    assert_eq!(
        log.acquisitions.saturating_sub(taken_before),
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
