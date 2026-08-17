//! Band 40 — running a `nomos work` verb, apart from choosing a platform or rendering the
//! answer.
//!
//! `OD-HOST-001` names the defect this crate answers: `nomos-cli`'s `work.rs` imported
//! `FileLock`, `StdFileSystem`, `StdProcessLauncher` and `SystemClock` from
//! `nomos-platform-std` directly, so choosing a platform, running a verb and rendering its
//! outcome were one body of code with no seam a second adapter could depend on without
//! taking all three.
//!
//! This crate is the middle one. [`command::WorkCommand`] is the request vocabulary — what
//! to do, independent of how it was spelled. [`Run`] does it: generic over the four traits
//! [`nomos_platform`] declares, so a composition root supplies its own filesystem, clock,
//! lock and process launcher (or the same ones `nomos-cli` does) and gets back
//! [`outcome::WorkOutcome`], a typed value naming what happened. Nothing here writes a line
//! of output or picks an exit code — that is the composition root's own next step, over a
//! value this crate handed it rather than a string.
//!
//! # What stays out
//!
//! Argument parsing stays where it was: a second adapter is not expected to parse `argv`,
//! only to construct the [`command::WorkCommand`] its own transport carries. And the
//! directory walk that finds this repository's already-published records — what
//! [`command::WorkCommand::Add`] needs to decide whether a declared amendment is honest —
//! stays with the composition root too: [`nomos_platform::FileSystem`] is read,
//! atomically-replace and exists, not a directory listing, and a general exclusion ledger
//! that learned to walk a source tree would be answering a question about this
//! repository's conventions rather than the platform's. `Run`'s `published` argument is
//! where that value arrives from outside.

#![forbid(unsafe_code)]

mod claim_request;
mod command;
mod ending_request;
mod outcome;
mod run;

#[cfg(test)]
mod tests;

pub use claim_request::ClaimRequest;
pub use command::WorkCommand;
pub use ending_request::EndingRequest;
pub use outcome::{BoardView, ShowView, WorkOutcome};
pub use run::Run;
