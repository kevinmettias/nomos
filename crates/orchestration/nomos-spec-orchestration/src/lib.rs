//! Band 40 — reading `nomos spec`'s store-free, store-reporting and store-projecting verbs,
//! apart from parsing arguments, choosing a corpus environment variable or rendering an
//! answer.
//!
//! `OD-HOST-002` names family 9's `SpecCommand` half as the one remaining command group with
//! no orchestration crate the way `WorkCommand` and, since `nomos-check-orchestration`,
//! `CheckCommand` each have one. This crate is that seam: the [`SpecCommand`] vocabulary and
//! its `*Request` types moved here verbatim from `nomos-cli::spec::command` and
//! `nomos-cli::spec::request`, the corpus-assembly plumbing every verb needs
//! ([`corpus::Assemble`], moved from `nomos-cli::corpus`) moved with them, and seven verbs
//! are now wired all the way through: [`SpecCommand::Profiles`] and [`SpecCommand::Sources`]
//! (increment 1); [`SpecCommand::Record`], [`SpecCommand::Table`] and
//! [`SpecCommand::Markdown`] (increment 2), the three read-only verbs that resolve an
//! identifier or an address against the store and hand back what they found, or a typed
//! refusal saying why nothing did; and [`SpecCommand::Render`] and [`SpecCommand::Freshness`]
//! (increment 3), the two verbs that place or read a governed output on disk.
//!
//! # Increment 3's decision: `Render` and `Freshness` are generic over `FileSystem`
//!
//! Both verbs' old homes in `nomos-cli::spec::verb::{render,freshness}` read and wrote a
//! rendered body and its sidecar through raw `std::fs`, unlike `nomos-work-orchestration`'s
//! `Run`, which is generic over [`nomos_platform`]'s traits throughout. This crate now
//! matches that seam for these two verbs specifically: [`run::Render`] and [`run::Freshness`]
//! take `filesystem: &F where F: nomos_platform::FileSystem`, and [`Run`] itself carries the
//! type parameter for all nine verbs even though seven of them never touch it — the same
//! shape `nomos_work_orchestration::Run` already has for `F`, `C`, `L` and `P`.
//!
//! The reasoning, not deferred: a rendered projection's body and its sidecar are two
//! single-file operations at paths this crate already knows before it opens either one --
//! exactly [`nomos_platform::FileSystem`]'s shape (read, atomically-replace, exists), and
//! nothing like the directory listing that port deliberately does not cover (why
//! [`corpus::Assemble`]'s own walk and `nomos-cli::work::Published_Records` both stay
//! client-side instead). Going through it is also a strict improvement, not a neutral
//! rewrite: the code it replaces wrote with `std::fs::write` after `std::fs::create_dir_all`,
//! a truncating write with a window in which the file is empty or half-written --
//! [`nomos_platform::FileSystem::Replace_Atomically`]'s own documentation names that as
//! exactly the defect it exists to close, and `nomos-platform-std`'s implementation already
//! creates missing parent directories on the way. `Preview` and `Commit` are deliberately
//! left out of this decision rather than folded in by default: their `--from` file is staged
//! by an author at a path this crate does not choose, which is a different shape from a
//! caller-known projection path, and settling whether that shape also wants `FileSystem` is
//! increment 4's decision to make with its own reasoning, not this one's to presume.
//!
//! # What stays out, and why
//!
//! The other two verbs (`Preview`, `Commit`) are unchanged: `nomos-cli::spec` still parses,
//! assembles a store for, and dispatches them entirely on its own, through the same
//! `spec/verb/editing.rs` functions it always has. [`SpecOutcome`] reserves a variant for
//! each of them ([`outcome::NotYetMigrated`]) so the vocabulary this crate will eventually
//! carry is visible now, rather than growing the enum's shape again as that increment lands.
//! Migrating their logic here is explicitly future work — one more increment, not this one.
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
pub use outcome::{
    FreshnessAnswer, FreshnessRefusal, NotYetMigrated, ProfileOutcome, RecordAnswer, RecordRefusal, RenderAnswer,
    RenderRefusal, SourcesAnswer, SpecOutcome, TableAnswer, TableRefusal, Verdict,
};
pub use request::{CommitRequest, EditRequest, FreshnessRequest, RecordRequest, RenderRequest, TableRequest};
pub use run::{Freshness, Markdown, Profiles, Record, Render, Run, Sources, Table};
