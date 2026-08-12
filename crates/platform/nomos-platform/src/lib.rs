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

// One module per port, each holding the trait and the types only that port's callers
// name. Flat, this level was thirteen files a reader had to sort into ports by opening
// them; the ports are the crate's whole structure and the tree now says so.
mod clock;
mod filesystem;
mod launcher;
mod lock;

pub use clock::{Clock, Timestamp};
pub use filesystem::{FileSystem, FileSystemError};
pub use launcher::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};
pub use lock::{CrossProcessLock, LockAcquisition, LockError, StaleTakeover};
