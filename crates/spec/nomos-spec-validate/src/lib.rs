//! Band 1 — the preservation rules.
//!
//! A run passes only when every declared rule ran to completion with no violations. An
//! internal error is a failure, never a skip: the absence of a validator has to be
//! indistinguishable from a failing one.

#![forbid(unsafe_code)]

mod preserve;
mod run;

pub use preserve::{
    ChangedWordingIsJustified, EveryBlockHasADisposition, EveryHeadingHasADisposition,
    EveryStatementTracesToSource, Registered,
};
pub use run::{
    DECLARED_RULES, Rule, RuleOutcome, RuleResult, Ruleset_Hash, Validate, ValidationRun,
    Violation,
};
