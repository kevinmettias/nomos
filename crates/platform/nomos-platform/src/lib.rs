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
//!
//! # Why `check-crate-split` reports this crate, and why it stays one
//!
//! The four ports never reference each other -- a clock has nothing to say to a lock --
//! so that check reads four groups sharing a manifest. The sealing is the point and it is
//! stated above: exactly one crate in this workspace names a platform dependency, and a
//! `tests/contract` assertion holds every crate below the host band to it. Four port
//! crates would be four places that rule has to be restated and checked.

#![forbid(unsafe_code)]

// One module per port, each holding the trait and the types only that port's callers
// name. Flat, this level was thirteen files a reader had to sort into ports by opening
// them; the ports are the crate's whole structure and the tree now says so.
mod clock;
mod file_system;
mod process_launcher;
mod cross_process_lock;

pub use clock::{Clock, Timestamp};
pub use file_system::{FileSystem, FileSystemError};
pub use process_launcher::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};
pub use cross_process_lock::{CrossProcessLock, LockAcquisition, LockError, StaleTakeover};
