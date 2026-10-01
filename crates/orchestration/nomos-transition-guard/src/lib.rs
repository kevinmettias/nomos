//! Zone: Application Service — refuses a commit or push that would carry material a named party
//! could claim, judged against a policy the user keeps outside every repository.
//!
//! `OD-POLICY-002` decides the whole of it; this crate is that record's model, matcher and
//! transitions, and `nomos-cli`'s `guard` group is the only host that composes it.
//!
//! # The policy, and what this crate never holds
//!
//! A [`PartyPolicy`] names a party, the e-mail domains that are its identities, rules of phrases
//! or ticket keys, the party's own repositories, and reasoned exceptions. It is read from JSON by
//! [`Party_Policy_From_Json`] and comes from a file the user keeps outside every repository.
//! Nothing in this workspace names a real party: tests and examples use invented parties at
//! `example.invalid`.
//!
//! A phrase is matched without regard to letter case unless its rule asks for `"case":
//! "exact"`, and without regard to spaces, dots, underscores or hyphens between its words, so
//! `"Northwind Parts"` also finds `northwind-parts` and `NorthwindParts`. A rule's `boundary` is
//! `word` unless it says `anywhere` or `word-start`. A ticket key `NW` finds `NW-12` and longer,
//! case-sensitive and bounded on both sides. There is no pattern language.
//!
//! # The transitions
//!
//! [`Judge_Commit_Being_Made`], [`Judge_Message`] and [`Judge_Push`] are the three moments git
//! runs a hook for; [`Scan`] audits a repository that is not yet guarded. Only what a moment adds
//! is judged. Each asks git only through [`GitReader`], which runs `git` through the
//! [`nomos_platform::ProgramLauncher`] port the caller hands it. [`Hook_Scripts`] is what
//! `nomos guard install` writes.
//!
//! # What this crate does not do
//!
//! It writes nothing: a refusal is returned for the host to print and is never stored, and no
//! exception can live in a repository. It adds no rule to `check` or the gate, reads no policy
//! from a repository, and edits no one's git configuration.

#![forbid(unsafe_code)]

mod git_reader;
mod guard_error;
mod hook_scripts;
mod judge;
mod matcher;
mod party_policy;
mod refusal;
mod transition;
mod unified_diff;

#[cfg(test)]
mod tests;

pub use git_reader::GitReader;
pub use guard_error::GuardError;
pub use hook_scripts::{Hook_Scripts, HookScript};
pub use judge::Judge;
pub use party_policy::{Boundary, Exception, LetterCase, MESSAGE_PLACE, OwnRepositories, PartyPolicy, PartyRule, Party_Policy_From_Json, Pattern, PolicyError};
pub use refusal::{IDENTITY_RULE, Place, Refusal, SHOWN_TEXT_LENGTH};
pub use transition::{Judge_Commit_Being_Made, Judge_Message, Judge_Push, Scan, ScanOutcome, Verdict};
