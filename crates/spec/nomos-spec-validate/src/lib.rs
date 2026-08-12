//! Band 1 — the preservation rules.
//!
//! A run passes only when every declared rule ran to completion with no violations. An
//! internal error is a failure, never a skip: the absence of a validator has to be
//! indistinguishable from a failing one.

#![forbid(unsafe_code)]

mod offending;
mod preserve;
mod rule;
mod run;
mod violation;

pub use preserve::Registered;
pub use rule::{Rule, RuleOutcome, RuleResult};
pub use run::{DECLARED_RULES, Validate, ValidationRun};
pub use violation::Violation;
