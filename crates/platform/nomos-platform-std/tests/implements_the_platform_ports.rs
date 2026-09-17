//! `nomos-platform-std` exists to do exactly one thing: satisfy `nomos_platform`'s ports
//! (`ProgramLauncher`, `FileSystem`, `Clock`, `FilesystemLock`) with ordinary operating-
//! system facilities. Every internal `#[cfg(test)]` module in this crate already proves its
//! own file's behavior, but only with private-item access; this file proves the same seam
//! through the public API a real caller has — construct a `nomos-platform-std` type, drive
//! it through `nomos_platform`'s own trait, and read the result back as `nomos_platform`'s
//! own types.

use nomos_platform::{Clock, Command, FilesystemLock, FileSystem, FileSystemError, LockError, ProgramLauncher};
use nomos_platform_std::{FileLock, StdFileSystem, StdProgramLauncher, SystemClock};
use std::path::Path;
use std::time::Duration;

/// How long the launcher waits for a child that exits immediately. Long enough that a real
/// process on a loaded machine is never killed for being slow, short enough that a launcher
/// which never reaps its child fails this test rather than hanging the suite.
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(10);

/// A lock older than this is treated as abandoned by a holder that died. A day, so that the
/// refusal this file asserts comes from the live holder rather than from staleness.
const STALE_AFTER: Duration = Duration::from_secs(86_400);

/// Remove a leftover `path` from an earlier run of this test.
///
/// Absent is the expected case and the only one this tolerates: a path that exists and cannot
/// be removed would make the test that follows it assert against a stale file, so it fails
/// here with the cause rather than there with a confusing mismatch.
fn Clear_Leftover(path: &Path)
{
    if let Err(cause) = std::fs::remove_file(path)
    {
        assert_eq!(cause.kind(), std::io::ErrorKind::NotFound, "{path:?} could not be cleared: {cause}");
    }
}

/// `StdProgramLauncher` against the real `ProgramLauncher` contract: a real child process,
/// judged through `nomos_platform::ExitOutcome`'s own `Is_Successful`/`Has_A_Verdict`.
#[test]
fn Test_Std_Program_Launcher_Should_Report_A_Real_Exit_Through_The_Real_Trait()
{
    let argv = if cfg!(windows)
    {
        vec!["cmd".to_owned(), "/C".to_owned(), "exit 0".to_owned()]
    }
    else
    {
        vec!["sh".to_owned(), "-c".to_owned(), "exit 0".to_owned()]
    };
    let command = Command::From_String_Arguments(argv, LAUNCH_TIMEOUT);

    let output = StdProgramLauncher.Run(&command).expect("a real, short-lived child process starts and exits");

    assert!(output.outcome.Is_Successful(), "a zero exit must satisfy nomos_platform's own success predicate");
    assert!(output.outcome.Has_A_Verdict(), "a zero exit is a real verdict, not a non-answer");
}

/// `StdFileSystem` against the real `FileSystem` contract: a real file, replaced
/// atomically and read back, and a missing one classified as `nomos_platform`'s own
/// `FileSystemError::NotFound`.
#[test]
fn Test_Std_File_System_Should_Round_Trip_A_Real_File_Through_The_Real_Trait()
{
    let filesystem = StdFileSystem;
    let path = std::env::temp_dir().join(format!("nomos-platform-std-seam-test-{}.txt", std::process::id()));
    Clear_Leftover(&path);

    filesystem.Replace_Atomically(&path, "seam-test-contents").expect("a fresh file can be written atomically");
    assert!(filesystem.Exists(&path));
    assert_eq!(filesystem.Read_To_String(&path).expect("the file just written can be read back"), "seam-test-contents");

    let missing = path.with_extension("does-not-exist");
    assert!(
        matches!(filesystem.Read_To_String(&missing), Err(FileSystemError::NotFound { .. })),
        "a path nothing wrote must classify as nomos_platform's own NotFound, not a generic failure"
    );

    Clear_Leftover(&path);
}

/// `SystemClock` against the real `Clock` contract: two real, back-to-back readings,
/// ordered and measured through `nomos_platform::Timestamp`'s own `Since`. No sleep: the
/// property under test -- the wall clock never runs backwards, and `Since` never goes
/// negative -- holds whether or not a real second elapsed between the two reads, and a
/// sleep here would only be betting on the scheduler rather than asserting the property.
#[test]
fn Test_System_Clock_Should_Produce_Real_Timestamps_The_Real_Trait_Can_Order()
{
    let clock = SystemClock;

    let first = clock.Now();
    let second = clock.Now();

    assert!(second.Unix_Seconds() >= first.Unix_Seconds(), "the wall clock must not run backwards between two readings");
    assert_eq!(first.Since(second), Duration::ZERO, "asking how long ago a later (or equal) reading was must never go negative");
}

/// `FileLock` against the real `FilesystemLock` contract: a lock acquired, then refused
/// to a second holder with `nomos_platform`'s own `LockError::Held`.
#[test]
fn Test_File_Lock_Should_Exclude_A_Second_Holder_Through_The_Real_Trait()
{
    let path = std::env::temp_dir().join(format!("nomos-platform-std-seam-test-{}.lock", std::process::id()));
    Clear_Leftover(&path);
    let lock = FileLock::At(&path);

    let held = lock.Acquire("seam-test-holder-a", Duration::ZERO, STALE_AFTER).expect("an uncontended lock is acquired cleanly");
    let refused = lock.Acquire("seam-test-holder-b", Duration::ZERO, STALE_AFTER);

    assert!(
        matches!(refused, Err(LockError::Held { .. })),
        "a lock already held through the real trait must refuse a second holder through the same trait"
    );

    drop(held);
    assert!(!path.exists(), "dropping the guard must release the lock");
}
