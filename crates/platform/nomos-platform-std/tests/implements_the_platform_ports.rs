//! `nomos-platform-std` exists to do exactly one thing: satisfy `nomos_platform`'s ports
//! (`ProcessLauncher`, `FileSystem`, `Clock`, `CrossProcessLock`) with ordinary operating-
//! system facilities. Every internal `#[cfg(test)]` module in this crate already proves its
//! own file's behavior, but only with private-item access; this file proves the same seam
//! through the public API a real caller has — construct a `nomos-platform-std` type, drive
//! it through `nomos_platform`'s own trait, and read the result back as `nomos_platform`'s
//! own types.

use nomos_platform::{Clock, Command, CrossProcessLock, FileSystem, FileSystemError, LockError, ProcessLauncher};
use nomos_platform_std::{FileLock, StdFileSystem, StdProcessLauncher, SystemClock};
use std::time::Duration;

/// `StdProcessLauncher` against the real `ProcessLauncher` contract: a real child process,
/// judged through `nomos_platform::ExitOutcome`'s own `Is_Successful`/`Has_A_Verdict`.
#[test]
fn Test_Std_Process_Launcher_Should_Report_A_Real_Exit_Through_The_Real_Trait()
{
    let argv = if cfg!(windows)
    {
        vec!["cmd".to_owned(), "/C".to_owned(), "exit 0".to_owned()]
    }
    else
    {
        vec!["sh".to_owned(), "-c".to_owned(), "exit 0".to_owned()]
    };
    let command = Command::New(argv, Duration::from_secs(10));

    let output = StdProcessLauncher.Run(&command).expect("a real, short-lived child process starts and exits");

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
    let _ = std::fs::remove_file(&path);

    filesystem.Replace_Atomically(&path, "seam-test-contents").expect("a fresh file can be written atomically");
    assert!(filesystem.Exists(&path));
    assert_eq!(filesystem.Read_To_String(&path).expect("the file just written can be read back"), "seam-test-contents");

    let missing = path.with_extension("does-not-exist");
    assert!(
        matches!(filesystem.Read_To_String(&missing), Err(FileSystemError::NotFound { .. })),
        "a path nothing wrote must classify as nomos_platform's own NotFound, not a generic failure"
    );

    let _ = std::fs::remove_file(&path);
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

/// `FileLock` against the real `CrossProcessLock` contract: a lock acquired, then refused
/// to a second holder with `nomos_platform`'s own `LockError::Held`.
#[test]
fn Test_File_Lock_Should_Exclude_A_Second_Holder_Through_The_Real_Trait()
{
    let path = std::env::temp_dir().join(format!("nomos-platform-std-seam-test-{}.lock", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let lock = FileLock::At(&path);

    let held = lock.Acquire("seam-test-holder-a", Duration::ZERO, Duration::from_secs(86_400)).expect("an uncontended lock is acquired cleanly");
    let refused = lock.Acquire("seam-test-holder-b", Duration::ZERO, Duration::from_secs(86_400));

    assert!(
        matches!(refused, Err(LockError::Held { .. })),
        "a lock already held through the real trait must refuse a second holder through the same trait"
    );

    drop(held);
    assert!(!path.exists(), "dropping the guard must release the lock");
}
