//! The temporary trees these tests write into, and the gate every one of them carries.
//!
//! Split out of `board.rs`, which says what an item, a territory and a holder are. Cleanup
//! machinery is a different subject: it is here so that file stays a readable statement of
//! the board rather than half fixture vocabulary and half filesystem bookkeeping.

use std::path::{Path, PathBuf};

/// A temporary repository that removes itself when the test holding it ends.
///
/// Every test here used to close with its own `remove_dir_all`, which is a line that only
/// runs when the test passes: a failed assertion unwinds straight past it. `Drop` runs on
/// the unwind too, so the tree is cleared exactly when the value goes out of scope and the
/// cleanup is no longer a step a test can forget or an assertion can skip.
pub(crate) struct Scratch(pub(crate) PathBuf);

impl Drop for Scratch
{
    fn drop(&mut self)
    {
        // `Drop` is infallible by construction: a panic raised while another panic unwinds
        // aborts the process, so a tree that will not go away is reported and left behind
        // rather than escalated into a crash that says nothing about the test.
        if let Err(error) = std::fs::remove_dir_all(&self.0)
        {
            eprintln!("the scratch tree {} outlived its test: {error}", self.0.display());
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

/// A tree named for the test that asked for it, cleared and recreated, with a gate in it.
///
/// The name carries this process's id, so two runs of the suite do not collide, while two
/// tests inside one run are distinguished by the name each passes — which is why every test
/// here names its own.
///
/// The tree is cleared first rather than simply created, because a previous test in this same
/// process may have left one behind under the same name; `create_dir_all` is idempotent, so a
/// failure to clear does not stop the test. It is reported instead, since what the test then
/// reads is somebody else's board.
pub(crate) fn Temporary_Directory(name: &str) -> Scratch
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-ledger-{name}-{}", std::process::id()));
    if let Err(error) = std::fs::remove_dir_all(&path)
    {
        eprintln!("the scratch tree {} was not cleared first: {error}", path.display());
    }
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    Write_Gate(&path);

    return Scratch(path);
}

/// Every tree these tests build is a repository with a gate, because finishing now reads
/// one and refuses when it cannot.
///
/// The lint step is `cargo --version` rather than the real clippy invocation. These tests
/// are about what a *predicate's* exit code does to an item; running a real workspace lint
/// in each of them would make the suite take minutes and would couple it to whatever the
/// workspace currently contains. What the derived step actually is, and that it comes from
/// the workflow rather than from a constant, is covered in `gate_covers_finish.rs`.
fn Write_Gate(directory: &Path)
{
    let workflows = directory.join(".github").join("workflows");
    std::fs::create_dir_all(&workflows).expect("test needs a workflow directory");
    std::fs::write(
        workflows.join("gate.yml"),
        "jobs:\n\
         \x20 gate:\n\
         \x20   steps:\n\
         \x20     - name: Lint\n\
         \x20       run: cargo --version\n",
    )
    .expect("test needs a workflow");
}
