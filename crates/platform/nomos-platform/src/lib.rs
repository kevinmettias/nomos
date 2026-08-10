//! Band 1p — the platform port. Traits only, no implementations.
//!
//! Everything Nomos needs from the machine underneath it passes through here, so that
//! swapping the implementation cannot recompile the bands above and so that exactly one
//! crate in the workspace names a platform dependency.
//!
//! # Why this exists before there is a second implementation
//!
//! The sibling xvpe workspace supplies most of these capabilities and Nomos will
//! eventually consume them. It is mid-refactor and does not currently compile, so
//! taking a path dependency on it today would make Nomos's buildability a function of
//! another product's work in progress. That is not a boundary problem, it is a schedule
//! problem, and it is worse.
//!
//! The seam exists now, with a std implementation behind it, precisely so that adopting
//! xvpe later is a new implementation rather than a refactor of every caller. A
//! `tests/contract` assertion holds the other half of the bargain: no crate below the
//! host band may name an `xvpe-*` dependency except the adapter that will implement
//! these traits.
//!
//! # Scope
//!
//! This crate declares what has a consumer today: [`Clock`], [`FileSystem`],
//! [`CrossProcessLock`] and [`ProcessLauncher`]. Blob storage, task hosting and
//! capability discovery are named in the architecture and are deliberately absent until
//! something needs them — a trait nothing implements and nothing calls is a claim about
//! the future, and this workspace has a rule against those.

#![forbid(unsafe_code)]

mod clock;
mod command;
mod exit_outcome;
mod file_system_error;
mod filesystem;
mod launcher;
mod lock;
mod lock_acquisition;
mod lock_error;
mod process_output;
mod stale_takeover;
mod timestamp;

pub use clock::Clock;
pub use command::Command;
pub use exit_outcome::ExitOutcome;
pub use file_system_error::FileSystemError;
pub use filesystem::FileSystem;
pub use launcher::ProcessLauncher;
pub use lock::CrossProcessLock;
pub use lock_acquisition::LockAcquisition;
pub use lock_error::LockError;
pub use process_output::ProcessOutput;
pub use stale_takeover::StaleTakeover;
pub use timestamp::Timestamp;
