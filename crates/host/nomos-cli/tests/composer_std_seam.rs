//! The `nomos_composer_std` seam `check-integration-coverage` found with no suite.
//!
//! `nomos_composer_std::FILE_SYSTEM` is the one filesystem every verb group in this binary
//! composes over `nomos_platform::FileSystem` — `work.rs`, `gate.rs`, `check.rs`,
//! `spec/verb/render.rs` and `spec/verb/freshness.rs` all choose it. Nothing under `tests/`
//! ever named the crate directly: every existing suite drives the compiled binary as a
//! subprocess and only ever sees the *effect* of that filesystem's calls (a ledger file
//! written, a rendered projection on disk), never the value itself.
//!
//! This uses the real composed filesystem directly and proves the two operations
//! nomos-cli's own production code depends on — an atomic replace that a plain read then
//! observes, and a `Read_To_String` that reports a missing file distinguishably rather than
//! panicking — then runs the real binary over a scratch tree so the same kind of file I/O
//! is proven to work through the shipped program too, not only through the adapter in
//! isolation.
//!
//! # This suite changed its subject and not its assertions
//!
//! It was `platform_std_seam.rs` until this crate stopped naming a platform implementation
//! directly. `FILE_SYSTEM` *is* the adapter it always exercised, reached through the
//! composer rather than by name, so all three facts below are the same facts about the same
//! implementation. Had the migration been wrong, these are the tests that would have said
//! so first, which is why the suite moved with the seam rather than being rewritten for it.

#[path = "support/mod.rs"]
mod support;

use nomos_platform::{FileSystem, FileSystemError};
use nomos_composer_std::FILE_SYSTEM;
use support::{Run, Tree};

/// A scratch path under the system temp directory, cleared before use and named for the
/// test that asked — the same convention `support::Tree` uses for a whole directory.
fn Scratch_File(name: &str) -> std::path::PathBuf
{
    let path = std::env::temp_dir().join(format!("nomos-cli-platform-std-seam-{name}-{}", std::process::id()));
    let _ignored = std::fs::remove_file(&path);
    return path;
}

/// `FILE_SYSTEM` is a plain, directly usable value — the same concrete adapter
/// `nomos-cli::work::Run` hands `nomos_work_orchestration::Run` and `nomos-cli::check::Run`
/// hands `nomos_check_orchestration::Run` as the platform's own `FileSystem`. A round trip
/// through `Replace_Atomically` and `Read_To_String` is the exact contract every one of
/// those call sites depends on: what was written is what comes back.
#[test]
fn Test_Std_File_System_Should_Round_Trip_A_Real_File()
{
    let path = Scratch_File("round-trip");
    let filesystem = FILE_SYSTEM;

    assert!(!filesystem.Exists(&path), "the scratch path must start absent");

    filesystem
        .Replace_Atomically(&path, "seam contents")
        .expect("a fresh path with a creatable parent must accept the write");

    assert!(filesystem.Exists(&path));
    assert_eq!(
        filesystem.Read_To_String(&path).expect("just written"),
        "seam contents"
    );

    let _ignored = std::fs::remove_file(&path);
}

/// The other half of the contract every caller of `FileSystem::Read_To_String` in this
/// crate leans on: a file that was never written is reported as `NotFound`, not conflated
/// with an empty one — `nomos-cli::work.rs` relies on exactly this to tell "no ledger yet"
/// from "an empty ledger".
#[test]
fn Test_Std_File_System_Should_Report_A_Missing_File_As_Not_Found()
{
    let path = Scratch_File("absent");
    let filesystem = FILE_SYSTEM;

    let error = filesystem.Read_To_String(&path).expect_err("nothing was ever written here");

    assert!(matches!(error, FileSystemError::NotFound { .. }), "{error}");
}

/// The same kind of file I/O, proven through the shipped binary: `nomos check` walks a real
/// scratch tree and must be able to read the file this test wrote into it, the same
/// operation `FILE_SYSTEM.Read_To_String` performs above -- production code's own choice
/// of adapter, exercised end to end rather than only in isolation.
#[test]
fn Test_The_Shipped_Binary_Should_Read_A_Real_File_Through_The_Same_Kind_Of_Access()
{
    let tree = Tree::New("platform-std-happy-path").With("subject.rs", "pub const TABLES: &[&str] = &[];\n");

    let ran = Run(&["check", "--root", &tree.Root()]);

    assert_eq!(ran.code, 0, "{}", ran.stdout);
    assert!(
        ran.stdout.contains("1 file(s) examined"),
        "the binary must have actually read the file this test wrote: {}",
        ran.stdout
    );
}
