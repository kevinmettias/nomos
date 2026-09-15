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

// Scoped to this file, and this file is the shared scaffolding three test binaries pull in
// through `#[path]`. Each of them uses a different subset of it, so a helper that only one
// root calls is genuinely unreachable when the other two are compiled and the lint fires
// there for code that is neither dead nor theirs. Nothing else lives in this file, so the
// blanket form allows only the helpers — the suites that include it keep their own lints.
#![allow(dead_code)]

use std::path::PathBuf;

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

/// The case a scratch board is named for, told apart from the suite it belongs to and the
/// ledger it is written with.
///
/// [`Board::New`] takes all three and all three are text, so a caller who wrote them the other
/// way round would name a directory after a ledger and write a ledger named for a suite, with
/// nothing to catch it. A caller still spells the value as a plain `&str`, which is why the
/// conversion lives here rather than at each of the suites sharing this file.
pub(crate) struct CaseName<'a>(&'a str);

impl<'a> From<&'a str> for CaseName<'a>
{
    fn from(name: &'a str) -> Self
    {
        return Self(name);
    }
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
    pub(crate) fn New<'a>(prefix: &str, name: impl Into<CaseName<'a>>, ledger: &str) -> Self
    {
        let name = name.into().0;
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
        use std::process::Command;

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
        return std::fs::read_to_string(self.Ledger())
            .expect("Board::New writes ledger.json at construction, so this board's file is on disk before any run reads it back");
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
