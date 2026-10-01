//! An in-memory tree that can be told to fail a particular write.
//!
//! The atomicity claim is about what happens when a write fails partway through a run, and
//! there is no honest way to make a real filesystem fail the second of three writes on
//! demand. So the port is substituted, which is the one thing `nomos-platform` exists for:
//! this implements the same four operations against a map, and fails whichever
//! `Replace_Atomically` call the test names.
//!
//! It models the port's own promise on failure — "the previous contents must remain intact"
//! — by not touching the map when it refuses. A fake that cleared the entry and then
//! reported an error would make the rollback look like it had worked when it had not.

use nomos_platform::{DeterminismStrength, FileSystem, FileSystemError, ReproducibilityScope, Strategy, TraceEquivalence};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// A whole tree, as a map from path to contents.
pub(crate) struct FakeTree
{
    files: RefCell<BTreeMap<PathBuf, String>>,
    /// Which `Replace_Atomically` calls refuse, counted from one across the whole run —
    /// rollback writes included, which is how a failing rollback is staged.
    failing_writes: BTreeSet<usize>,
    writes: Cell<usize>,
}

/// Answers from fixed data it holds itself, so its outputs reproduce byte for byte.
impl Strategy for FakeTree
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl FakeTree
{
    /// An empty tree that fails no write.
    pub(crate) fn New() -> Self
    {
        return Self { files: RefCell::new(BTreeMap::new()), failing_writes: BTreeSet::new(), writes: Cell::new(0) };
    }

    /// The same tree, holding one more file.
    pub(crate) fn With(self, path: &Path, contents: &str) -> Self
    {
        self.files.borrow_mut().insert(path.to_path_buf(), contents.to_owned());

        return self;
    }

    /// The same tree, refusing the named writes.
    pub(crate) fn Failing(mut self, calls: &[usize]) -> Self
    {
        self.failing_writes = calls.iter().copied().collect();

        return self;
    }

    /// Everything the tree holds, for comparing a run's before against its after.
    pub(crate) fn Snapshot(&self) -> BTreeMap<PathBuf, String>
    {
        return self.files.borrow().clone();
    }

    /// One file's contents, or nothing if the tree does not hold it.
    pub(crate) fn Contents(&self, path: &Path) -> Option<String>
    {
        return self.files.borrow().get(path).cloned();
    }

    /// How many writes were attempted, refused ones included.
    pub(crate) fn Attempted_Writes(&self) -> usize
    {
        return self.writes.get();
    }
}

impl FileSystem for FakeTree
{
    fn Read_To_String(&self, path: &Path) -> Result<String, FileSystemError>
    {
        return self
            .files
            .borrow()
            .get(path)
            .cloned()
            .ok_or_else(|| return FileSystemError::NotFound { path: path.display().to_string() });
    }

    fn Replace_Atomically(&self, path: &Path, contents: &str) -> Result<(), FileSystemError>
    {
        let call = self.writes.get().saturating_add(1);
        self.writes.set(call);
        if self.failing_writes.contains(&call)
        {
            // Nothing is inserted, which is what the port promises: a failed replacement
            // leaves the previous contents intact.
            return Err(FileSystemError::Denied {
                path: path.display().to_string(),
                cause: format!("this tree was told to refuse write {call}"),
            });
        }
        self.files.borrow_mut().insert(path.to_path_buf(), contents.to_owned());

        return Ok(());
    }

    fn Exists(&self, path: &Path) -> bool
    {
        return self.files.borrow().contains_key(path);
    }

    fn Remove_File(&self, path: &Path) -> Result<(), FileSystemError>
    {
        return self
            .files
            .borrow_mut()
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| return FileSystemError::NotFound { path: path.display().to_string() });
    }
}
