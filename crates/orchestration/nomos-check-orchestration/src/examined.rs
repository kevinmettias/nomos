//! What a `nomos check` run produced.
//!
//! Every variant is a real branch the pipeline already had -- `check.rs`, `facts.rs` and
//! `composition.rs` used to decide one of these and then immediately write a line and pick
//! an [`ExitCode`], and a second adapter had no way to ask "what happened" without also
//! taking the writing and the exit code. This is the seam's whole point, matching
//! `nomos_work_orchestration::WorkOutcome`: a caller that pattern-matched rendered text out
//! of it would have reintroduced the coupling this crate exists to remove.
//!
//! [`ExitCode`]: <https://doc.rust-lang.org/std/process/struct.ExitCode.html>

mod check_outcome;
mod claim;
mod fact_read;
mod supporting_fact_trail;
mod supporting_facts;

pub use check_outcome::CheckOutcome;
pub use claim::{Claim, Claim_Of};
pub use fact_read::FactRead;
pub(crate) use fact_read::Reduced;
pub use supporting_fact_trail::SupportingFactTrail;
pub use supporting_facts::SupportingFacts;

/// How much of the world this run actually saw.
///
/// Two denominators and not one. "0 findings over 400 files" and "0 findings over 400
/// files none of which produced a fact" are different claims. Moved verbatim from
/// `nomos-cli::check::report` -- see that module's history for why the second number
/// exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Examined
{
    /// Files the walk read.
    pub files: usize,
    /// Files a syntax fact was materialized for.
    pub facts: usize,
}
