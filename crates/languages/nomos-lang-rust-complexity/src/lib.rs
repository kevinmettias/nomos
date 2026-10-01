//! Zone: Provider — each Rust function's cyclomatic complexity, for
//! `nomos.cap.metric.complexity`.
//!
//! # What is counted
//!
//! The cyclomatic number: one path through a function, plus one for every place control can take
//! another. Here that is each `if` (an `else if` is its own `if`), each `while` and `while let`,
//! each `for`, each `match` arm after the first and each arm guard, each `&&` and `||` (both
//! short-circuit, so each is a branch), each `?` (an early return), and each `let ... else`. A
//! `loop` adds nothing: it has one way in and its exits are the `break`s inside it, which sit
//! under a condition that is already counted.
//!
//! A closure is counted into the function that contains it, because its branches are that
//! function's to read. A function declared inside another is its own subject, and its branches
//! are not its parent's.
//!
//! # What is not, and why that is stated rather than discovered
//!
//! A macro invocation is an unexpanded token stream to `syn`, so a branch inside `assert!`,
//! `matches!` or `vec![if ... ]` is not seen, and neither is a function a macro defines. Every
//! value here is therefore a floor: exact for the syntax as written, low wherever a macro hides
//! a branch, and never high. [`Declared_Guarantee`] says so on the completeness axis, and
//! `tests/guarantee.rs` holds the case that shows it.

#![forbid(unsafe_code)]

mod complexity_fact_production;
mod complexity_reading;
mod fact_context;
mod guarantee;
mod materialization;
mod parse_failure;
mod production;
mod walk;

pub use complexity_fact_production::ComplexityFactProduction;
pub use complexity_reading::{ComplexityReading, Read_Complexity};
pub use fact_context::FactContext;
pub use guarantee::{Declared_Guarantee, PROVIDER, Provider_Offer};
pub use materialization::Materialization;
pub use parse_failure::ParseFailure;
pub use production::Materialize_Complexity_Fact;
