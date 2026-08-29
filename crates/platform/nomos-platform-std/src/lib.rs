//! Band 1p — the std implementation of the platform port.
//!
//! Ordinary operating-system facilities, with two pieces that are worth reading rather
//! than skimming: the atomic file replace, and the cross-process lock.

#![forbid(unsafe_code)]

mod system_clock;
mod file_lock_guard;
mod std_file_system;
mod std_process_launcher;
mod file_lock;

pub use system_clock::SystemClock;
pub use std_file_system::StdFileSystem;
pub use std_process_launcher::StdProcessLauncher;
pub use file_lock_guard::FileLockGuard;
pub use file_lock::FileLock;
