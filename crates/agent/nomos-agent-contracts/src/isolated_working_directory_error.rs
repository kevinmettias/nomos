//! A freshly created, empty working directory under the system temp root, for a
//! subprocess-based agent dispatch to run inside.
//!
//! Shared by every `AgentExecutor`/`ModelBackend` adapter that isolates its subprocess
//! this way -- `nomos-agent-executor-claude-code` and `nomos-model-backend-ollama` each
//! built this same shape independently before it moved here, so the invariant it
//! carries -- never this repository's own tree, never colliding between two calls in
//! the same process -- is enforced once rather than reimplemented per adapter.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Why an isolated working directory could not be created.
#[derive(Debug)]
pub struct IsolatedWorkingDirectoryError
{
    /// The path that could not be created.
    pub path: PathBuf,
    /// What the operating system said.
    pub cause: std::io::Error,
}

impl core::fmt::Display for IsolatedWorkingDirectoryError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(
            formatter,
            "could not create an isolated working directory at {}: {}",
            self.path.display(),
            self.cause
        );
    }
}

/// The one owner of the process-lifetime sequence [`Isolated_Working_Directory`] mixes into
/// every directory name. Nothing outside [`Next`](Self::Next) ever touches the atomic it
/// wraps, so "who can write this" has a single, named answer instead of a bare global
/// anyone could reach into.
struct SequenceCounter(AtomicU64);

impl SequenceCounter
{
    const fn New() -> Self
    {
        return Self(AtomicU64::new(0));
    }

    /// A number no other call in this process has been given before.
    fn Next(&self) -> u64
    {
        // atomic-ordering: allow: only used to give two calls in this process different numbers;
        // nothing else synchronizes on it or reads memory ordered by this counter.
        return self.0.fetch_add(1, Ordering::Relaxed);
    }
}

/// A freshly created, empty directory under the system temp root, named from `prefix`,
/// this process's id and a per-process counter shared by every caller -- so two calls in
/// the same process, whatever their prefix, never collide, and nothing here depends on
/// the wall clock having advanced.
///
/// # Errors
///
/// [`IsolatedWorkingDirectoryError`] if the directory could not be created.
pub fn Isolated_Working_Directory(prefix: &str) -> Result<PathBuf, IsolatedWorkingDirectoryError>
{
    static COUNTER: SequenceCounter = SequenceCounter::New();

    let sequence = COUNTER.Next();
    let directory = std::env::temp_dir().join(format!("{prefix}-{}-{sequence}", std::process::id()));

    std::fs::create_dir_all(&directory).map_err(|cause| IsolatedWorkingDirectoryError { path: directory.clone(), cause })?;

    return Ok(directory);
}
