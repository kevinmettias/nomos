//! A change to certain paths names the packages that enumerate them: what a repository
//! declares a predicate must carry, and why an item carrying less is refused.
//!
//! `OD-GATE-036` decided it, on five landings that passed their own predicate and left `HEAD`
//! red in a package none of those predicates ran. An item whose territory reaches a path the
//! repository declares must carry a predicate naming the arguments the declaration requires,
//! or one that satisfies it alone, and `add` and `widen` refuse one that does not. Authoring
//! time rather than finish time, so the author sees the cost while choosing the item's
//! timeout.
//!
//! # What this crate does and does not know
//!
//! It knows the declaration's shape and compares tokens. It does not know which repository it
//! serves or what any token means to the tool that runs it: the rule is declared by the
//! repository in [`PREDICATE_COVERAGE`] and read by the caller, the same way the caller reads
//! which records a repository has published, and handed to [`crate::FileLedger::Add`] and
//! [`crate::FileLedger::Widen`] inside [`crate::RepositoryDeclarations`]. The rule is never
//! written into the ledger, which serves repositories that have no such file and are
//! untouched by it.

mod coverage_refusal;
mod predicate_coverage;

pub use coverage_refusal::CoverageRefusal;
pub use predicate_coverage::{CoverageRule, PREDICATE_COVERAGE, PredicateCoverage};
