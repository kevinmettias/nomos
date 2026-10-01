//! `nomos guard` — refuses a commit or push that would carry material a named party could
//! claim, judged against a policy the user keeps outside every repository.
//!
//! `OD-POLICY-002` decides it and `nomos_transition_guard` is its whole judgment; this group
//! only composes that crate with this binary's standard launcher, environment and file system,
//! and renders the answer.
//!
//! # The verbs
//!
//! `pre-commit`, `commit-msg <file>` and `pre-push <remote> [url]` are what git runs as hooks,
//! with git's own arguments; `scan` audits a repository that is not guarded yet; `install`
//! writes the hook scripts. Every verb but `install` reads the policies named by `--policy`,
//! given once per party, or the one named by `NOMOS_PARTY_POLICY` when no flag is given. Each
//! policy is judged on its own, and a refusal from any of them refuses.
//!
//! # What this group never does
//!
//! It writes no report, cache or baseline, and prints a refusal only to standard error,
//! because a refusal repeats what the private policy matched (`OD-POLICY-002` decision 5). It
//! never edits git configuration: `install` prints the one line that points git at the scripts.

mod exit_code;
mod parsing;
mod run;

#[cfg(test)]
mod tests;

pub use exit_code::ExitCode;
pub use parsing::{Guard_Command_From_String_Arguments, GuardCommand, Verb};
pub use run::{GuardContext, POLICY_VARIABLE, Run};
