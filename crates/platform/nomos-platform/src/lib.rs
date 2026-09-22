//! Zone: Substrate — the platform port. Traits only, no implementations.
//!
//! Everything Nomos needs from the machine underneath it passes through here, so that
//! swapping the implementation cannot recompile the bands above and so that exactly one
//! crate in the workspace names a platform dependency.
//!
//! # What the seam bought, now that it has been used
//!
//! This crate was written before XVPE could be depended on, so that adopting it later
//! would be a new implementation behind an existing seam rather than a refactor of
//! every caller. That is what happened, and the seam is why the move did not touch a
//! caller: [`ProgramLauncher`]'s design went *down* into `xvpe-subprocess-execution`
//! on 2026-09-10, with `nomos-platform-xvpe` bridging this workspace's launchers onto
//! it. That trait is unchanged and its 33 implementors never learned.
//!
//! That is `OD-PLATFORM-003`: Nomos is an application over that engine, so a
//! domain-neutral capability sitting up here is unreachable by everything down there.
//! The rule that once said only `nomos-platform-xvpe` may name `xvpe-` is retired with
//! that record — naming `xvpe-` is ordinary now, and the `tests/contract` assertion
//! that enforced it was deleted rather than widened, so no rule here reads as a
//! boundary while enforcing nothing.
//!
//! [`Timestamp`] went down the same way on 2026-09-11 and came back on 2026-09-21,
//! under `OD-ROADMAP-005` decision 4. Ownership of the type was never the objection:
//! a *port* crate that cannot compile without the engine it is a port to has a
//! dependency running the wrong way through the seam, and every band above this one
//! inherits it. So this crate names no `xvpe-` dependency, and the seam paid for the
//! return trip exactly as it paid for the outward one — no use-site learned either
//! time, and the nine serde fields that write a timestamp into the work ledger still
//! name [`timestamp_serde`], because the wire format never moved. What the crossing
//! kept is untouched by this: it stays adopted, it stays pinned, and
//! `nomos-platform-xvpe` is still the adapter.
//!
//! # Scope
//!
//! This crate declares what has a consumer today: [`Clock`], [`FileSystem`],
//! [`FilesystemLock`], [`ProgramLauncher`] and [`Environment`]. Blob storage, task
//! hosting and capability discovery are named in the architecture and are deliberately
//! absent until something needs them — a trait nothing implements and nothing calls is a
//! claim about the future, and this workspace has a rule against those.
//!
//! [`Environment`] is the one added by that rule rather than despite it. Three providers
//! were injecting a [`ProgramLauncher`] and then reading `std::env` past it to decide what
//! the launched command was called, so the seam existed and was being stepped around;
//! `P86` measured the three identical reads. Its two operations are the two that had
//! callers, and `std::env::args` is not among them for the same reason the absent ports
//! above are absent: every production call site of it is a `main.rs`.
//!
//! [`Clock`] stays this workspace's own rather than becoming XVPE's
//! `WallClockStrategy`. The two are the same design, operation for operation, down to
//! the two `Timestamp` declarations — but XVPE additionally requires every strategy
//! surface to declare its determinism, and adopting that here would mean touching every
//! implementor for no behavioural change. That is the same trade `nomos-platform-xvpe`
//! already made for the launcher, and the answer is the same: bridge at the crossing,
//! not at the port.
//!
//! # Why `check-crate-split` reports this crate, and why it stays one
//!
//! The five ports never reference each other -- a clock has nothing to say to a lock --
//! so that check reads five groups sharing a manifest. The sealing is the point and it is
//! stated above: exactly one crate in this workspace names a dependency on the machine
//! underneath it. Five port crates would be five manifests for that one fact, and
//! [`Environment`] — the next port added after that sentence was written — would have had
//! to pick between them.

#![forbid(unsafe_code)]

// One module per port, each holding the trait and the types only that port's callers
// name. Flat, this level was thirteen files a reader had to sort into ports by opening
// them; the ports are the crate's whole structure and the tree now says so.
//
// A port's error type sits here rather than inside the port's own folder, because the
// name it is published under already carries the port: `file_system/file_system_error.rs`
// repeats its parent folder, and `file_system/error.rs` declaring `Error` needs an alias
// to be published at all. See each file's own header.
mod clock;
mod file_system;
mod file_system_error;
mod program_launcher;
mod filesystem_lock;
mod environment;
mod environment_error;

// The determinism vocabulary the four ports declare in, re-exported so that an
// implementor names it through the crate whose trait it is implementing. Every implementor
// already depends on this crate -- that is what implementing its port means -- so this is
// the difference between one import and a new dependency edge in each of the eighteen
// crates that stand something up behind a port. The authority is still `nomos-contracts`;
// this is a re-export, not a second copy.
pub use nomos_contracts::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};

pub use clock::{Clock, Timestamp, timestamp_serde};
pub use file_system::FileSystem;
pub use file_system_error::FileSystemError;
pub use program_launcher::{Command, ExitOutcome, ProgramLauncher, ProgramOutput};
pub use filesystem_lock::{FilesystemLock, LockAcquisition, LockError, StaleTakeover};
pub use environment::Environment;
pub use environment_error::EnvironmentError;
