//! The std backend set, assembled once.
//!
//! A composition root needs a platform. Before this crate the only way to have one was to
//! name `nomos-platform-std`'s implementations directly, and so every root did — measured
//! 2026-09-12, `nomos-api` at 34 production sites across 24 files and `nomos-cli` at 22
//! across 11, restating the same choices per host and often per handler.
//!
//! `OD-HOST-001` already settled that naming a concrete provider in a composition root is
//! not itself the defect, and this crate does not reopen that. What was missing is
//! anything for a root to name it *through*: a place where "this workspace runs on the
//! ordinary operating system" is written once, so a host expresses *which platform* rather
//! than *which implementation of each port*.
//!
//! # What this crate is, and what it is not
//!
//! It **selects**. It names `nomos-platform-std` and re-exports `nomos-platform`'s own port
//! vocabulary beside it, so a caller needs a dependency on neither to use what is here.
//! That is the whole job.
//!
//! It **orchestrates nothing**. There is no verb here, no sequence, and no type that exists
//! to be rendered. A caller wanting a `nomos work` verb run still calls
//! `nomos-work-orchestration`; this crate only supplies what that call must be given. The
//! distinction is `OD-HOST-001`'s three-crate split, and a composer is a fourth thing beside
//! it rather than a second copy of the middle one: a library, never a composition root,
//! deciding nothing a host has not asked it for.
//!
//! It **holds no state and opens nothing**. Every value here is a unit struct or is built
//! from a path the caller supplies, so naming a platform cannot fail and cannot touch the
//! machine until the caller uses it.
//!
//! # Why this is not the only composer there will be
//!
//! `std` is the backend *set*, not the composition. A second set — an in-memory platform for
//! a harness, or the XVPE-backed launcher `nomos-platform-xvpe` already implements — becomes
//! a sibling crate exposing these same names, and a host swaps one import rather than
//! rewriting every site that named a concrete type. That substitution is the reason this
//! crate exists; until a second set arrives it is deduplication, and this says so rather
//! than claiming an option nothing has exercised.

#![forbid(unsafe_code)]

use std::path::PathBuf;

// The concrete types this set selects, one per file. Each is separately replaceable — a
// second composer swaps the file a name resolves to without touching any other member — and
// each is re-exported here so that the path a caller names is the same one it named before
// they were split apart.
mod clock_type;
mod environment_type;
mod file_system_type;
mod launcher_type;
mod lock_type;

pub use clock_type::ClockType;
pub use environment_type::EnvironmentType;
pub use file_system_type::FileSystemType;
pub use launcher_type::LauncherType;
pub use lock_type::LockType;

// The port vocabulary, re-exported so a caller that takes this crate needs no separate
// dependency on `nomos-platform` to name the traits its own signatures are generic over.
// The authority is still `nomos-platform`; this is a re-export and not a second
// declaration, the same thing `nomos-platform` itself already does for the determinism
// vocabulary that `nomos-contracts` declares.
pub use nomos_platform::{Clock, FilesystemLock, Environment, FileSystem, ProgramLauncher};
pub use nomos_platform::{EnvironmentError, FileSystemError, LockError};
pub use nomos_platform::{Command, ExitOutcome, ProgramOutput};
pub use nomos_platform::{LockAcquisition, StaleTakeover, Timestamp, timestamp_serde};

// The determinism vocabulary, for the same reason and by the same authority chain: a host
// reading `FILE_SYSTEM`'s own `STRENGTH` to decide whether a run may claim reproducibility
// should not need a third dependency to name the answer it gets back.
pub use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

/// The filesystem this platform reads and writes through.
pub const FILE_SYSTEM: FileSystemType = nomos_platform_std::StdFileSystem;

/// The clock this platform reads the time from.
pub const CLOCK: ClockType = nomos_platform_std::SystemClock;

/// The launcher this platform runs subprocesses through.
pub const LAUNCHER: LauncherType = nomos_platform_std::StdProgramLauncher;

/// The environment this platform reads variables and the working directory from.
pub const ENVIRONMENT: EnvironmentType = nomos_platform_std::StdEnvironment;

/// A cross-process lock over `path`.
///
/// A function rather than a constant because a lock is the one member of this set that is
/// not a unit struct: it is *about* a particular file, and which file is the caller's
/// decision rather than this platform's. Nothing is created or opened here — the file is
/// touched when the lock is taken, not when it is named.
#[must_use]
pub fn Lock_At(path: impl Into<PathBuf>) -> LockType
{
    return nomos_platform_std::FileLock::At(path);
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Each constant must really be an implementation of the port it is offered as.
    /// Asserted through functions bounded by the port rather than by calling an inherent
    /// method, so this stops compiling if a constant is ever swapped for something that
    /// merely has the right shape.
    #[test]
    fn Test_Every_Constant_Should_Satisfy_The_Port_It_Is_Offered_As()
    {
        fn Strength_Of<Selected: Strategy>(_selected: &Selected) -> DeterminismStrength
        {
            return Selected::STRENGTH;
        }
        fn As_File_System<Selected: FileSystem>(selected: &Selected) -> DeterminismStrength
        {
            return Strength_Of(selected);
        }
        fn As_Clock<Selected: Clock>(selected: &Selected) -> DeterminismStrength
        {
            return Strength_Of(selected);
        }
        fn As_Launcher<Selected: ProgramLauncher>(selected: &Selected) -> DeterminismStrength
        {
            return Strength_Of(selected);
        }
        fn As_Environment<Selected: Environment>(selected: &Selected) -> DeterminismStrength
        {
            return Strength_Of(selected);
        }

        // Every one reaches the real machine, so every one promises nothing. A selection
        // whose members disagreed here would be a platform that reproduces in some of its
        // parts, which is not a claim any caller could act on.
        assert_eq!(As_File_System(&FILE_SYSTEM), DeterminismStrength::None);
        assert_eq!(As_Clock(&CLOCK), DeterminismStrength::None);
        assert_eq!(As_Launcher(&LAUNCHER), DeterminismStrength::None);
        assert_eq!(As_Environment(&ENVIRONMENT), DeterminismStrength::None);
    }

    /// `Lock_At` names a file without touching it. A composer that created something on the
    /// way past would make merely *describing* a platform a side effect, and a caller that
    /// built one to inspect it would leave a lock file behind.
    #[test]
    fn Test_Lock_At_Should_Not_Create_Anything()
    {
        let path = std::env::temp_dir().join("nomos-composer-std-unwritten.lock");
        let _ignored = std::fs::remove_file(&path);

        let _lock = Lock_At(path.clone());

        assert!(!path.exists(), "{} was created by naming it", path.display());
    }
}
