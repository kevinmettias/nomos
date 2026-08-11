//! Band 1 — the preservation rules.
//!
//! A run passes only when every declared rule ran to completion with no violations. An
//! internal error is a failure, never a skip: the absence of a validator has to be
//! indistinguishable from a failing one.

#![forbid(unsafe_code)]

mod changed_wording_is_justified;
mod every_block_has_a_disposition;
mod every_statement_traces_to_source;
mod offending;
mod preserve;
mod rule;
mod rule_outcome;
mod rule_result;
mod run;
mod violation;

pub use preserve::Registered;
pub use rule::Rule;
pub use rule_outcome::RuleOutcome;
pub use rule_result::RuleResult;
pub use run::{DECLARED_RULES, Validate, ValidationRun};
pub use violation::Violation;
