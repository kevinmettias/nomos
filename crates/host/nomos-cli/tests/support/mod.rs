//! Shared harness for the crate-boundary seam tests under `tests/*_seam.rs`.
//!
//! `nomos-cli` is a `[[bin]]`-only crate -- there is no `src/lib.rs`, so nothing under
//! `tests/` can link against it as a library and reach its internal modules. The only
//! public surface this crate has is the compiled binary itself: its argv, its stdout,
//! its stderr, its exit code. Every seam test drives that surface, the same way
//! `tests/check_command.rs` already does (`CARGO_BIN_EXE_nomos`, no nested `cargo` build --
//! `OD-GATE-002` records why a test that shells out to `cargo` from inside `cargo test`
//! deadlocks on the target-directory lock).
//!
//! # Why this file declares no tests of its own
//!
//! `tests/list_tells_the_truth.rs`'s own doc comment explains the same shape this file
//! reuses: a bare `mod x;` in a file directly under `tests/` resolves to `tests/x.rs`, which
//! cargo would then build as a further, independent test target. Living under
//! `tests/support/` keeps this module out of cargo's automatic top-level discovery while
//! `#[path = "support/mod.rs"] mod support;` still lets every seam file reach it.

use std::path::PathBuf;
use std::process::Command;

/// The binary under test, as cargo built it.
pub const NOMOS: &str = env!("CARGO_BIN_EXE_nomos");

/// What one run of the binary exited with, and what it said on each stream.
///
/// Named rather than a tuple, so that a caller reading one member is reading a name and
/// not a position -- the same reason `tests/check_command.rs`'s own `Ran` is named.
pub struct Ran
{
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Runs the real `nomos` binary and returns what it exited with and what it said.
pub fn Run(arguments: &[&str]) -> Ran
{
    let finished = Command::new(NOMOS)
        .args(arguments)
        .output()
        .expect("the binary cargo just built must be runnable");

    return Ran {
        code: finished.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&finished.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&finished.stderr).into_owned(),
    };
}

/// One file's path inside a scratch tree, told apart from the text written into it.
///
/// [`Tree::With`] takes both and both are text, so a caller who wrote them the other way
/// round would create a directory named after one file's contents and write a path into a
/// file -- compiling, and asserting about nothing. Callers still spell the value as a plain
/// `&str`, or as the `&String` a `format!` produces, so the conversions live here rather
/// than at each of the seam tests sharing this file.
pub struct FileName<'a>(&'a str);

impl<'a> From<&'a str> for FileName<'a>
{
    fn from(name: &'a str) -> Self
    {
        return Self(name);
    }
}

impl<'a> From<&'a String> for FileName<'a>
{
    fn from(name: &'a String) -> Self
    {
        return Self(name.as_str());
    }
}

/// A scratch tree under the system temp directory, removed when it is dropped.
pub struct Tree
{
    pub root: PathBuf,
}

impl Tree
{
    /// Makes one, named for the test that asked, so a failure leaves an identifiable
    /// directory behind rather than an anonymous one.
    pub fn New(name: &str) -> Self
    {
        let root = std::env::temp_dir().join(format!("nomos-cli-seam-{name}-{}", std::process::id()));

        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory under the temp directory");

        return Self { root };
    }

    /// Writes one file into the tree, creating parent directories as needed.
    pub fn With<'a>(self, name: impl Into<FileName<'a>>, text: &str) -> Self
    {
        let name = name.into().0;
        let path = self.root.join(name);
        if let Some(parent) = path.parent()
        {
            std::fs::create_dir_all(parent).expect("parent directories are creatable");
        }
        std::fs::write(path, text).expect("writing into a directory just created");
        return self;
    }

    /// This tree's root as a `--root` argument.
    pub fn Root(&self) -> String
    {
        return self.root.display().to_string();
    }
}

impl Drop for Tree
{
    fn drop(&mut self)
    {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}
