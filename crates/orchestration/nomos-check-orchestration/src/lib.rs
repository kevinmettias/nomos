//! Zone: Application Service — composing the capability registry, ingesting already-walked source into
//! facts, and judging it, apart from choosing a platform, walking a tree or rendering the
//! answer.
//!
//! `OD-HOST-002` named the defect this crate answers: `OD-HOST-001` built
//! `nomos-work-orchestration` as the seam between choosing a platform, running a verb and
//! rendering its outcome, but three more state families around `nomos check` had no such
//! seam. `nomos-cli::check::composition::Registered`/`Resolved_Configuration` were the only
//! way to obtain a resolved capability registry (family 2); `nomos-cli::check::facts::
//! Prepare` built the fact store fresh per call and dropped it when `check::Run` returned,
//! stateless only because the CLI happens to exit (family 3); and `nomos_contracts::Finding`
//! was reachable only by running `nomos check` itself, inside `nomos-cli::check::report`
//! (family 4). A second surface wanting any of these had two options: reimplement
//! `check.rs`, or accept whatever `nomos-cli` computed as fact.
//!
//! This crate is the middle one, the same shape `nomos-work-orchestration` is for `work`.
//! [`check_command::CheckCommand`] is the request vocabulary. [`Run`] does the composing and
//! the judging: it takes source already walked and the build variant already read, and hands
//! back [`examined::CheckOutcome`], a typed value naming the pipeline's real branches.
//! Nothing here writes a line of output or picks an exit code.
//!
//! # What stays out
//!
//! The directory walk stays in `nomos-cli::check::sources` — not because the port cannot
//! enumerate a directory: `OD-PLATFORM-002` gave [`nomos_platform::FileSystem`] a
//! `Read_Directory`, and that primitive is deliberately one level rather than a traversal, so
//! a recursive walk is composed by a caller. `nomos-cli::check::sources::Walked_Sources`
//! carries that reason and this line routes to it rather than restating it. What this binary
//! was compiled as
//! (`nomos-cli::check::composition::Host_Variant`) stays there too: it is read through
//! `env!`, which resolves against the crate that calls it, so a build variant computed
//! inside this crate would describe this library's own compilation rather than the binary
//! that called it. Argument parsing, exit codes and the text a person reads all stay in
//! `nomos-cli::check` — a process convention and a rendering step, not a verb outcome.
//!
//! `tests/integration/src/context.rs`'s own rendering of a registry is a second, real
//! duplication of [`composition::Resolved_Configuration`] that `OD-HOST-002` already names
//! and does not ask this crate to remove.

#![forbid(unsafe_code)]

mod check_command;
mod declared_rules;
mod composition;
mod examined;
mod facts;
mod package_conformance;
mod rule_resolution;
mod rule_resolution_error;
mod run_context;

#[cfg(test)]
mod tests;

pub use check_command::CheckCommand;
pub use declared_rules::Declared_Rules;
pub use composition::{Registered, Resolved_Configuration};
pub use examined::{Claim, Claim_Of, CheckOutcome, Examined, FactRead, SupportingFactTrail, SupportingFacts};
pub use package_conformance::{Check_Package_Conformance, DeclaredPackage, PACKAGE_CONFORMANCE};
pub use rule_resolution::{Resolve_Rules, RuleResolution};
pub use rule_resolution_error::RuleResolutionError;
pub use run_context::{Composed_Rules, RuleReassessmentCache, Run, RunContext, Run_Reassessing};
