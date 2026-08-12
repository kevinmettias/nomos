//! Why a submission was not accepted.

use core::fmt;

use crate::Failure;

/// Why a submission was not accepted.
///
/// Carries every failure rather than the first. Reporting one at a time makes a form with six
/// holes take six refusals and teaches the rule set by exhaustion, which is the cost
/// `OD-LEDGER-007` measured for a refusal that stops more than it needed to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal
{
    /// Which submission was refused.
    pub submission: String,
    /// Every rule it failed.
    pub failures: Vec<Failure>,
}

impl fmt::Display for Refusal
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        writeln!(
            formatter,
            "{} is refused, and nothing was stored. {} rule(s) failed:",
            self.submission,
            self.failures.len()
        )?;

        for failure in &self.failures
        {
            writeln!(
                formatter,
                "  {} failed {}: {}",
                failure.field, failure.rule, failure.remedy
            )?;
        }

        return Ok(());
    }
}
