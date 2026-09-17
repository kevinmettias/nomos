//! Zone: Substrate — the XVPE implementation of the platform port.
//!
//! Anticipated in the workspace manifest since the port was written, and absent
//! until XVPE had the engine code to implement it against. It does now: the
//! agent dispatch this workspace built — an isolated working directory, an
//! allow-list granting nothing, a spend ceiling, a wall bound and an answer
//! constrained by a schema — moved down into XVPE on 2026-09-10, because nomos
//! is an application over that engine and a general capability sitting up here
//! was unreachable by everything down there.
//!
//! # What this crate is, and is not
//!
//! It is **one adapter**: any of this workspace's `ProgramLauncher`s, seen as the
//! launcher surface XVPE's own adapters take.
//!
//! It is **not** a replacement for `nomos-platform`'s port. The two are the same
//! design — they were the same code — but XVPE's trait additionally requires a
//! declared determinism, and adopting it here would mean touching 33
//! implementors and 559 use-sites across twenty crates for no behavioural
//! change. Everything in this workspace keeps naming its own port; anything
//! reaching into XVPE wraps its launcher once, here.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod xvpe_launcher;

pub use xvpe_launcher::XvpeLauncher;
