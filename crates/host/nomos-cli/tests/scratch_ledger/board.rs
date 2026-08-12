//! A ledger on disk, the binary run against it, and what the run said.
//!
//! Three suites beside this directory drive `nomos work` against a board they wrote
//! themselves, and the writing was the same five statements in each of them: a
//! process-unique directory under the temporary root, cleared, created, and a
//! `ledger.json` put in it. A reader had to diff the three to know they agreed, and a fix
//! applied to one left the other two wrong.
//!
//! What each suite still says for itself is the *content* of the board — the items, the
//! claims, the key no build declares. That is the case each suite is written about, so it
//! stays in the suite and only the scaffolding is shared.
//!
//! Declared through `#[path]` by each root rather than as `tests/board.rs`. A bare `mod` in
//! a test binary root resolves against `tests/`, which cargo would then build as a fourth
//! test target; a directory holding no `main.rs` is not one cargo scans. `OD-GATE-002` and
//! the split at `2cad614` are the precedent.

#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;

/// The binary under test, as cargo built it for this suite.
///
/// Never by shelling out to `cargo`: a nested `cargo test` deadlocks on the target lock,
/// which `OD-GATE-002` records.
pub(crate) const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

/// What one `nomos work` run said, and what it exited with.
///
/// Named rather than a pair. The same two values were spelled `(String, i32)` in one of
/// these files and `(i32, String)` in the next, which is exactly the swap a name makes
/// impossible and a type does not.
pub(crate) struct Ran
{
    pub(crate) said: String,
    pub(crate) code: i32,
}

/// A scratch board on disk, removed when the test that made it ends.
pub(crate) struct Board
{
    root: PathBuf,
}

impl Board
{
    /// A board carrying `ledger`, under a directory named for `prefix` and `name`.
    ///
    /// The process id is part of the directory name because these are separate test
    /// binaries and cargo runs them at the same time; `prefix` separates the suites and
    /// `name` separates the cases within one.
    pub(crate) fn New(prefix: &str, name: &str, ledger: &str) -> Self
    {
        let root =
            std::env::temp_dir().join(format!("nomos-{prefix}-{name}-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");
        std::fs::write(root.join("ledger.json"), ledger).expect("a scratch ledger");

        return Self { root };
    }

    /// Runs `nomos work …` against this board, returning what it said and what it exited
    /// with.
    pub(crate) fn Work(&self, arguments: &[&str]) -> Ran
    {
        let output = Command::new(NOMOS)
            .arg("work")
            .args(arguments)
            .env("NOMOS_WORK_DIR", &self.root)
            .output()
            .expect("the binary runs");

        return Ran {
            said: String::from_utf8_lossy(&output.stdout).into_owned(),
            code: output.status.code().unwrap_or(-1),
        };
    }

    /// What `nomos work …` printed, for a caller that is not asserting on the exit code.
    pub(crate) fn Said(&self, arguments: &[&str]) -> String
    {
        return self.Work(arguments).said;
    }

    /// The ledger file exactly as it stands, bytes and all.
    ///
    /// Bytes rather than a type: a refusal that exits non-zero while quietly rewriting the
    /// file would satisfy every assertion about exit codes and would be the defect intact.
    pub(crate) fn Bytes(&self) -> Vec<u8>
    {
        return std::fs::read(self.Ledger()).expect("the ledger is readable");
    }

    /// The ledger file as text, for asserting on what was written rather than on what was
    /// said.
    pub(crate) fn Written(&self) -> String
    {
        return std::fs::read_to_string(self.Ledger()).expect("readable");
    }

    fn Ledger(&self) -> PathBuf
    {
        return self.root.join("ledger.json");
    }
}

impl Drop for Board
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}
