//! Whether a `nomos check` run reached a judgment about everything it touched.

use nomos_contracts::Finding;

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
