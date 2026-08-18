//! Band 40 — reading `nomos spec`'s two store-free and store-reporting verbs, apart from
//! parsing arguments, choosing a corpus environment variable or rendering an answer.
//!
//! `OD-HOST-002` names family 9's `SpecCommand` half as the one remaining command group with
//! no orchestration crate the way `WorkCommand` and, since `nomos-check-orchestration`,
//! `CheckCommand` each have one. This crate is that seam's first increment: the
//! [`SpecCommand`] vocabulary and its `*Request` types move here verbatim from
//! `nomos-cli::spec::command` and `nomos-cli::spec::request`, the corpus-assembly plumbing
//! every verb needs ([`corpus::Assemble`], moved from `nomos-cli::corpus`) moves with them,
//! and the two verbs that touch no store or touch one only to describe it —
//! [`SpecCommand::Profiles`] and [`SpecCommand::Sources`] — are wired all the way through.
//!
//! # What stays out, and why
//!
//! The other seven verbs (`Record`, `Table`, `Render`, `Freshness`, `Markdown`, `Preview`,
//! `Commit`) are unchanged: `nomos-cli::spec` still parses, assembles a store for, and
//! dispatches them entirely on its own, through the same `spec/verb/*.rs` functions it
//! always has. [`SpecOutcome`] reserves a variant for each of them
//! ([`outcome::NotYetMigrated`]) so the vocabulary this crate will eventually carry is
//! visible now, rather than growing the enum's shape three more times as each increment
//! lands. Migrating their logic here is explicitly future work — three more increments,
//! not this one.
//!
//! Argument parsing (`spec/parsing.rs`), exit-code mapping (`spec/exit_code.rs`) and text
//! rendering (`spec/reporting.rs`, and `Note_Absences`'s own decision about *whether* to
//! print an absence) all stay in `nomos-cli`, the same discipline
//! `nomos-check-orchestration` already set: this crate returns typed data and writes
//! nothing.

#![forbid(unsafe_code)]

pub mod corpus;
mod command;
mod outcome;
mod request;
mod run;

#[cfg(test)]
mod tests;

pub use command::SpecCommand;
pub use outcome::{NotYetMigrated, SourcesAnswer, SpecOutcome};
pub use request::{CommitRequest, EditRequest, FreshnessRequest, RecordRequest, RenderRequest, TableRequest};
pub use run::{Profiles, Run, Sources};
