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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The two adapters that share this function (`nomos-agent-executor-claude-code`,
    /// `nomos-model-backend-ollama`) each cover it transitively, through their own
    /// same-named private wrapper -- real coverage, but under an address that names their
    /// wrapper and not this function. This is the direct test for the shared primitive
    /// itself.
    #[test]
    fn Test_Isolated_Working_Directory_Should_Create_A_Fresh_Empty_Directory()
    {
        let directory = Isolated_Working_Directory("nomos-agent-contracts-test").expect("creates a real directory");

        assert!(directory.is_dir());
        let entries: Vec<_> = std::fs::read_dir(&directory).expect("reads the directory").collect();
        assert!(entries.is_empty(), "a freshly created isolated directory must start empty");

        // Best-effort cleanup: the system temp root is reclaimed independently of this
        // test, and a failure to remove this directory changes nothing the assertions
        // above already established.
        let _ = std::fs::remove_dir(&directory);
    }

    #[test]
    fn Test_Isolated_Working_Directory_Should_Never_Collide_Across_Two_Calls()
    {
        let first = Isolated_Working_Directory("nomos-agent-contracts-test").expect("creates a real directory");
        let second = Isolated_Working_Directory("nomos-agent-contracts-test").expect("creates a real directory");

        assert_ne!(first, second);

        // Best-effort cleanup: the system temp root is reclaimed independently of this
        // test, and a failure to remove either directory changes nothing the assertion
        // above already established.
        let _ = std::fs::remove_dir(&first);
        let _ = std::fs::remove_dir(&second);
    }
}
