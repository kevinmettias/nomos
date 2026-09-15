//! What one baselined `rule`/`subject` scope accepted, against what a run actually found.

use crate::BaselineAllowance;
use nomos_contracts::{RuleId, SubjectId};

/// What one baselined `rule`/`subject` scope accepted, against what a run actually found.
///
/// The unit `OD-GATE-030` reports in, and deliberately not a per-finding verdict. The record's
/// own words: the honest report is about the population -- what it accepted, what it observed,
/// and the excess.
///
/// # What this is evidence of, and what it is not
///
/// An `observed` above an `allowed` proves that at least `observed - allowed` of the
/// occurrences present cannot belong to the population that was adopted. That is a counting
/// argument and it is exact. It is **not** evidence about any particular occurrence, and
/// `observed` at or below `allowed` is **not** evidence that the adopted occurrences persisted
/// -- a scope that accepted five and observes five is equally consistent with the same five
/// persisting and with all five having been fixed while five different violations appeared.
/// Continuity, persistence and reintroduction stay unresolved until historical identity
/// evidence exists, and `IdentityTransitionKind` is the vocabulary that increment reuses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaselinePopulation
{
    /// The rule whose occurrences this scope counts.
    pub rule: RuleId,
    /// The subject whose occurrences this scope counts.
    pub subject: SubjectId,
    /// The path the entry that matched this scope was written with, when a declared file is
    /// where it came from.
    ///
    /// Copied off [`crate::BaselineDebt::declared_path`] rather than recomputed, and for the
    /// reason it exists there: a report names a scope by what its author typed, and `subject`
    /// is a digest no reader can get back to their own configuration from. Never identity --
    /// matching stays on `rule`/`subject`, and two populations here can differ in this field
    /// while being the one scope those two spellings denote.
    pub declared_path: Option<String>,
    /// What the declared entry accepted.
    pub allowed: BaselineAllowance,
    /// How many occurrences this run found in the scope.
    pub observed: u32,
}

impl BaselinePopulation
{
    /// How far past its allowance this population is -- zero when it is within one, and zero
    /// when the entry named no allowance at all.
    ///
    /// Derived rather than stored, so it cannot come to disagree with the two numbers it is
    /// computed from. A stored excess is a third fact that can be wrong on its own.
    #[must_use]
    pub const fn Excess(&self) -> u32
    {
        return match self.allowed
        {
            BaselineAllowance::Unbounded => 0,
            // Saturating because an excess is a count and not a difference: a population
            // inside its allowance is over by nothing, which is what zero says, and this
            // workspace denies arithmetic that could wrap here rather than trusting the order.
            BaselineAllowance::AtMost(allowed) => self.observed.saturating_sub(allowed),
        };
    }

    /// Whether this scope holds more occurrences than it accepted.
    #[must_use]
    pub const fn Is_Exceeded(&self) -> bool
    {
        return self.Excess() > 0;
    }
}
