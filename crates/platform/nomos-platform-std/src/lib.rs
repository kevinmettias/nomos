//! Band 1p — the std implementation of the platform port.
//!
//! Ordinary operating-system facilities, with two pieces that are worth reading rather
//! than skimming: the atomic file replace, and the cross-process lock.

#![forbid(unsafe_code)]

mod clock;
mod file_lock_guard;
mod filesystem;
mod launcher;
mod lock;

pub use clock::SystemClock;
pub use filesystem::StdFileSystem;
pub use launcher::StdProcessLauncher;
pub use file_lock_guard::FileLockGuard;
pub use lock::FileLock;
