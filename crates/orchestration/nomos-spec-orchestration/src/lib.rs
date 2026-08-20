//! Band 40 — running every `nomos spec` verb, and `nomos request submit`, apart from parsing
//! arguments, choosing a corpus environment variable or rendering an answer.
//!
//! `OD-HOST-002` named family 9's `SpecCommand` half as the one remaining command group with
//! no orchestration crate the way `WorkCommand` and, since `nomos-check-orchestration`,
//! `CheckCommand` each have one. This crate is that seam, built over four increments and now
//! complete: the [`SpecCommand`] vocabulary and its `*Request` types moved here verbatim from
//! `nomos-cli::spec::command` and `nomos-cli::spec::request`, the corpus-assembly plumbing
//! every verb needs ([`corpus::Assemble`], moved from `nomos-cli::corpus`) moved with them,
//! and all nine verbs are wired all the way through: [`SpecCommand::Profiles`] and
//! [`SpecCommand::Sources`] (increment 1); [`SpecCommand::Record`], [`SpecCommand::Table`]
//! and [`SpecCommand::Markdown`] (increment 2), the three read-only verbs that resolve an
//! identifier or an address against the store and hand back what they found, or a typed
//! refusal saying why nothing did; [`SpecCommand::Render`] and [`SpecCommand::Freshness`]
//! (increment 3), the two verbs that place or read a governed output on disk; and
//! [`SpecCommand::Preview`] and [`SpecCommand::Commit`] (increment 4), `D-129`'s authoring
//! round trip -- stage an edit, preview what it would change, and commit it as a transaction
//! against the store.
//!
//! # A second command group, answered beside `SpecCommand` rather than folded into it
//!
//! `OD-HOST-002` named a fourth control-command group, `request::Command`, that family 9 left
//! open. `OD-HOST-005` decided its one verb, `Submit`, is not a fourth peer crate at some band
//! above 40: reading `nomos-cli::request` whole showed its real work was already three calls
//! into this crate's own domain -- [`corpus::Assemble`], `nomos-spec-store`'s
//! `Accept_Submission`, and, through `--into`, the same `subject-dossier` render
//! [`SpecCommand::Render`] already wraps. [`Submit`] is that verb, added here at no new band
//! and no new crate -- but it is not a tenth [`SpecCommand`] variant, because that record's
//! own resolution is explicit that "a `nomos request submit` invocation is not a `nomos spec`
//! verb by the CLI's own naming": `SpecCommand` and [`SpecOutcome`] stay exactly the nine
//! verbs `nomos spec` answers, and [`Submit`] is a sibling of [`Run`] over its own
//! [`SubmitRequest`] and [`SubmitAnswer`]/[`SubmitRefusal`] pair instead. `nomos request
//! submit` is still its own CLI surface -- `nomos-cli::request` keeps its own argument
//! parsing, exit codes and text rendering, exactly as `nomos-cli::spec` does for `SpecCommand`
//! -- but the verb it dispatches to lives here now, the same way every `SpecCommand` variant's
//! real work already does.
//!
//! # Increment 3's decision: `Render` and `Freshness` are generic over `FileSystem`
//!
//! Both verbs' old homes in `nomos-cli::spec::verb::{render,freshness}` read and wrote a
//! rendered body and its sidecar through raw `std::fs`, unlike `nomos-work-orchestration`'s
//! `Run`, which is generic over [`nomos_platform`]'s traits throughout. This crate matched
//! that seam for these two verbs specifically: [`run::Render`] and [`run::Freshness`] take
//! `filesystem: &F where F: nomos_platform::FileSystem`, and [`Run`] itself carries the type
//! parameter for all nine verbs even though five of them never touch it — the same shape
//! `nomos_work_orchestration::Run` already has for `F`, `C`, `L` and `P`.
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
//! creates missing parent directories on the way.
//!
//! # Increment 4's decision: `Preview` and `Commit` are generic over `FileSystem` too, and
//! vacating a rename is not
//!
//! Increment 3 deliberately left this open rather than presumed: `Preview`'s and `Commit`'s
//! `--from` file is staged by an author at a path this crate does not choose, which it
//! flagged as "a different shape" from a projection's own caller-known output path.
//! Measured against what the port actually is, the shape difference does not hold up.
//! [`nomos_platform::FileSystem::Read_To_String`] reads one whole file and does not care who
//! chose the path — an author-staged `--from` file is still exactly one file, read whole, the
//! same operation `Render` already performs on paths it computes itself. [`run::preview`]
//! goes through it: [`run::Preview`] takes `filesystem: &F` and reads `--from` with
//! [`nomos_platform::FileSystem::Read_To_String`]. The benefit is the same one increment 3
//! already banked for `Render` — a caller can hand `Preview` a fake `FileSystem` and exercise
//! a missing `--from` without touching a real disk.
//!
//! `Commit` follows for the same reason on its write half: the record's committed bytes land
//! at `into.join(&report.path)`, a caller-known root joined with a store-produced relative
//! path — exactly [`run::Render`]'s own shape, through
//! [`nomos_platform::FileSystem::Replace_Atomically`] for the same durability reason. See
//! [`run::commit`] for that half.
//!
//! What does *not* follow is vacating a rename's old path. Deletion is not a shape question
//! at all — [`nomos_platform::FileSystem`] declares three operations (read, atomically
//! replace, exists) and its own documentation calls that "small on purpose", so there is no
//! port method for this crate to route through even if the reasoning above otherwise applied.
//! Extending `nomos-platform` is out of this crate's territory, so [`run::commit`] calls
//! `std::fs::remove_file` directly for that one step — the same arrangement
//! [`corpus::Assemble`]'s own directory walk already has for an operation the port does not
//! cover, not a departure from the pattern.
//!
//! Argument parsing (`spec/parsing.rs`), exit-code mapping (`spec/exit_code.rs`) and text
//! rendering (`spec/reporting.rs`, and `Note_Absences`'s own decision about *whether* to
//! print an absence) all stay in `nomos-cli`, the same discipline
//! `nomos-check-orchestration` already set: this crate returns typed data and writes only
//! the files a verb's own job is to write ([`run::Render`]'s projection,
//! [`run::Commit`]'s committed record) — never a line of commentary about either one.

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
    CommitAnswer, CommitRefusal, CommitRefusalError, CommitRefusalKind, FreshnessAnswer, FreshnessRefusal,
    PreviewRefusal, ProfileOutcome, RecordAnswer, RecordRefusal, RenderAnswer, RenderRefusal, Reproduction,
    SourcesAnswer, SpecOutcome, SubmitAnswer, SubmitRefusal, TableAnswer, TableRefusal, VacateOutcome, Vacated,
    Verdict,
};
pub use request::{
    CommitRequest, EditRequest, FreshnessRequest, RecordRequest, RenderRequest, SubmitRequest, TableRequest,
};
pub use run::{Commit, Freshness, Markdown, Preview, Profiles, Record, Render, Run, Sources, Submit, Table};
