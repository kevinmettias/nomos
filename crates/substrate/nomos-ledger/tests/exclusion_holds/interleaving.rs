//! The harness that makes a lost update happen on purpose.
//!
//! Two writers that merely run at the same time reproduce a lost update by luck, and a test
//! that reproduces by luck goes green on a slower machine while the defect is still there.
//! What is needed is the interleaving itself, so one writer is held between its read and its
//! write while the other completes.
//!
//! The harness is read by `concurrency.rs`, which is where the claims it supports are made.

use crate::board::*;

/// How long the harness lets the second writer run before it releases the first one.
///
/// This is not a timing assumption about the machine. Under the arrangement these tests
/// exist to hold, the second writer *cannot* finish while the first one is between its read
/// and its write, so this wait is expected to expire — it is the bound on how long the
/// harness waits to learn that. Without the arrangement the second writer finishes in well
/// under a millisecond and the wait ends immediately, so the failing case is fast and the
/// passing case pays this once.
pub(crate) const SECOND_WRITER_LIMIT: Duration = Duration::from_millis(750);

/// A one-way gate: opened once, and waited on by whoever needs to know it opened.
///
/// A [`std::sync::Barrier`] would be shorter and cannot express the wait that is *meant* to
/// time out, which is the whole of what the second half of this harness observes.
pub(crate) struct Gate
{
    open: Mutex<bool>,
    changed: Condvar,
}

impl Gate
{
    fn New() -> Self
    {
        return Self {
            open: Mutex::new(false),
            changed: Condvar::new(),
        };
    }

    fn Open(&self)
    {
        let mut open = self.open.lock().expect("the harness never panics under this lock");
        *open = true;
        self.changed.notify_all();
    }

    fn Wait(&self)
    {
        let open = self.open.lock().expect("the harness never panics under this lock");
        let _held = self
            .changed
            .wait_while(open, |open| return !*open)
            .expect("the harness never panics under this lock");
    }

    /// Waits up to `limit`, and reports whether the gate opened within it.
    fn Opened_Within(&self, limit: Duration) -> bool
    {
        let open = self.open.lock().expect("the harness never panics under this lock");
        let (_held, timing) = self
            .changed
            .wait_timeout_while(open, limit, |open| return !*open)
            .expect("the harness never panics under this lock");
        return !timing.timed_out();
    }
}

/// A filesystem that holds one thread still between its read of the ledger and its write.
///
/// # Why the seam is here and not in the store
///
/// Two writers that merely run at the same time reproduce a lost update by luck, and a test
/// that reproduces by luck is one that goes green on a slower machine while the defect is
/// still there. What is needed is the interleaving itself: one writer's read must be known
/// to have happened before the other writer's whole operation, and its write must be known
/// to happen after.
///
/// [`FileLedger`] already takes the filesystem it reads through, for a reason its own doc
/// comment gives — a caller reaching for `std::fs` would put the store beyond a test's
/// control. That injection point is enough, so nothing test-only is added to the store: this
/// is an ordinary [`FileSystem`] that happens to stop after handing back the bytes.
pub(crate) struct Interleaving
{
    ledger: PathBuf,
    /// The thread that gets held, registered by that thread itself.
    held: Mutex<Option<ThreadId>>,
    /// Opened when the held thread has read the document it is about to write back.
    read: Gate,
    /// Opened by the harness when the held thread may proceed to its write.
    resume: Gate,
    /// Whether the hold has already happened, so it happens once rather than per read.
    stopped: AtomicBool,
}

impl Interleaving
{
    fn Over(ledger: PathBuf) -> Self
    {
        return Self {
            ledger,
            held: Mutex::new(None),
            read: Gate::New(),
            resume: Gate::New(),
            stopped: AtomicBool::new(false),
        };
    }

    /// Registers the calling thread as the one to hold.
    fn Hold_This_Thread(&self)
    {
        let mut held = self.held.lock().expect("the harness never panics under this lock");
        *held = Some(std::thread::current().id());
    }

    fn Holds(&self, path: &Path) -> bool
    {
        if path != self.ledger
        {
            return false;
        }

        let held = *self.held.lock().expect("the harness never panics under this lock");

        return held == Some(std::thread::current().id());
    }

    /// Whether the hold ever happened. The harness asserts this: a run in which the seam
    /// never fired proves nothing about interleaving, however green it is.
    fn Stopped(&self) -> bool
    {
        return self.stopped.load(Ordering::SeqCst);
    }
}

impl FileSystem for &Interleaving
{
    fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
    {
        let text = StdFileSystem.Read_To_String(path);

        // After the read, never before it. The point of the hold is that this thread is
        // carrying a snapshot of the document that somebody else is about to change.
        if self.Holds(path) && !self.stopped.swap(true, Ordering::SeqCst)
        {
            self.read.Open();
            self.resume.Wait();
        }

        return text;
    }

    fn Replace_Atomically(&self, path: &Path, contents: &str) -> Result<(), FileSystemError>
    {
        return StdFileSystem.Replace_Atomically(path, contents);
    }

    fn Exists(&self, path: &Path) -> bool
    {
        return StdFileSystem.Exists(path);
    }
}

pub(crate) type InterleavedLedger<'shared> =
    FileLedger<&'shared Interleaving, &'shared FixedClock, FileLock>;

pub(crate) fn Ledger_Over<'shared>(
    filesystem: &'shared Interleaving,
    directory: &Path,
    clock: &'shared FixedClock,
) -> InterleavedLedger<'shared>
{
    return FileLedger::At(
        directory.join("ledger.json"),
        filesystem,
        clock,
        FileLock::At(directory.join("ledger.lock")),
    );
}

/// Runs two writers against one ledger with `first` held between its read and its write.
///
/// The order is fixed rather than raced. `second` does not start until `first` has read, and
/// `first` does not write until `second` has either finished or been kept waiting for
/// [`SECOND_WRITER_LIMIT`]. Both outcomes are legitimate and they are what the two
/// arrangements look like from outside: without exclusion `second` completes inside the
/// window and `first` then writes over it; with exclusion `second` is still waiting for the
/// lock when the window closes, and it reads `first`'s write when it finally gets in.
// `Send` on both closures is required by `Interleaved`, which runs each through
// `std::thread::scope(..).spawn(..)` (waived in suppressions.json, since check-closure-bounds
// is not one of the safety-critical checks an in-code marker still argues under this
// repository's safety-only policy).
pub(crate) fn Two_Writers(
    name: &str,
    items: Vec<LedgerItem>,
    first: impl FnOnce(&mut InterleavedLedger<'_>) + Send,
    second: impl FnOnce(&mut InterleavedLedger<'_>) + Send,
) -> LedgerDocument
{
    let directory = Contended(name, items);
    let filesystem = Interleaving::Over(directory.join("ledger.json"));

    let over = Harness {
        shared: &filesystem,
        finished: &Gate::New(),
        directory: &directory,
    };

    Interleaved(over, first, second);
    assert!(
        filesystem.Stopped(),
        "the seam never fired, so nothing was interleaved and this run proves nothing"
    );

    return Written(&directory);
}

/// The apparatus both writers share: the filesystem carrying the seam, the gate the second
/// writer opens when it is through, and the tree they are both writing.
#[derive(Clone, Copy)]
pub(crate) struct Harness<'a>
{
    shared: &'a Interleaving,
    finished: &'a Gate,
    directory: &'a Path,
}

/// The scope itself: `second` starts once `first` has read, and `first` writes once `second`
/// has either finished or been kept waiting for [`SECOND_WRITER_LIMIT`].
// `Send` on both closures is required by `std::thread::scope(..).spawn(..)` below (waived in
// suppressions.json, since check-closure-bounds is not one of the safety-critical checks an
// in-code marker still argues under this repository's safety-only policy).
pub(crate) fn Interleaved(
    over: Harness<'_>,
    first: impl FnOnce(&mut InterleavedLedger<'_>) + Send,
    second: impl FnOnce(&mut InterleavedLedger<'_>) + Send,
)
{
    std::thread::scope(|scope| {
        scope.spawn(move || {
            over.shared.Hold_This_Thread();
            let mut ledger = Ledger_Over(over.shared, over.directory, &AT_NOW);
            first(&mut ledger);
        });

        over.shared.read.Wait();

        scope.spawn(move || {
            let mut ledger = Ledger_Over(over.shared, over.directory, &AT_NOW);
            second(&mut ledger);
            over.finished.Open();
        });

        over.finished.Opened_Within(SECOND_WRITER_LIMIT);
        over.shared.resume.Open();
    });
}

/// A board for the interleaving tests, saved once before either writer starts.
pub(crate) fn Contended(name: &str, items: Vec<LedgerItem>) -> Scratch
{
    let directory = Temp_Dir(name);
    Ledger_At(&directory, &AT_NOW)
        .Save(&Document(items))
        .expect("a fresh ledger is valid");

    return directory;
}

/// What is actually on disk once both writers have finished — the only thing that settles
/// whether a write was lost, since each writer was told its own succeeded.
pub(crate) fn Written(directory: &Path) -> LedgerDocument
{
    return Ledger_At(directory, &AT_NOW).Load().expect("readable");
}

pub(crate) fn Holder_Of(document: &LedgerDocument, id: &str) -> Option<String>
{
    return document
        .items
        .iter()
        .find(|item| return item.id == ItemId::New(id))
        .and_then(|item| return item.claim.as_ref())
        .map(|claim| return claim.holder.clone());
}
