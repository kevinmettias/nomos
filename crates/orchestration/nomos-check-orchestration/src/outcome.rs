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

use nomos_capability::RegistryError;
use nomos_contracts::Finding;

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

/// Whether this run reached a judgment about everything it touched.
///
/// Not implied by zero blocking findings. [`Claim::Incomplete`] is a fact about the run's
/// reach, not about severity -- `OD-COMPLETENESS-004` records that this stays a fact
/// reported in the text rather than a fact the exit code carries. Moved verbatim from
/// `nomos-cli::check::report`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Claim
{
    /// No subject fell into a debt or agent-required state. Deliberate absences --
    /// `Applicability::NotApplicable` and `Applicability::ConfigurationDisabled` -- do
    /// not break this, for the same reason `Applicability::Is_Coverage_Debt` excludes
    /// them: a decision is not a gap.
    Complete,
    /// At least one subject fell into `Applicability::Is_Coverage_Debt` or
    /// `Applicability::Requires_Agent`. The run did not reach a judgment about it, and
    /// that is a different claim from reaching one and finding it clean.
    Incomplete,
}

impl core::fmt::Display for Claim
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(match self
        {
            Self::Complete => "complete",
            Self::Incomplete => "incomplete",
        });
    }
}

/// The roll-up verdict, read off `Applicability`'s own predicates rather than re-derived
/// from a per-bucket count -- one classification, asked once, so a change to what counts as
/// debt cannot drift between a caller's own breakdown and this one.
#[must_use]
pub fn Claim_Of(findings: &[Finding]) -> Claim
{
    let unjudged = findings.iter().any(|finding| {
        return finding.applicability.Is_Coverage_Debt() || finding.applicability.Requires_Agent();
    });

    return if unjudged { Claim::Incomplete } else { Claim::Complete };
}

/// What a `nomos check` run produced.
pub enum CheckOutcome
{
    /// The root does not exist or is not a directory, or the walked source could not be
    /// ingested as a workspace state. Nothing was judged.
    Unreadable,
    /// This build's own capability registry is self-contradictory -- a defect in the
    /// composition, not in the tree being checked.
    Contradictory(RegistryError),
    /// The walk found no source under the root.
    NoSource,
    /// Source was found but no syntax fact was materialized for any of it, so no mirror
    /// claim could be resolved. The same lie as [`CheckOutcome::NoSource`], one layer in.
    NoFacts
    {
        /// Files the walk read, none of which produced a fact.
        files: usize,
    },
    /// The rules ran over a real fact store.
    Judged
    {
        /// What the rules found.
        findings: Vec<Finding>,
        /// How much of the world this run actually saw.
        examined: Examined,
        /// Whether this run reached a judgment about everything it touched.
        claim: Claim,
    },
}
