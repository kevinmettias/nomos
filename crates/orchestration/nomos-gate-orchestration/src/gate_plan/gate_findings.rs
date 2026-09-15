//! [`GateFindings`], the finding groups [`super::GateRunResult`] carries.

use crate::BaselineAllowance;
use nomos_contracts::{Finding, RuleId, SubjectId};

/// A run's findings, grouped by why each one does or does not block the build -- named so a
/// caller reads `findings.blocking_findings` and the rest by field rather than telling four
/// same-shaped lists apart only by which struct they sat in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateFindings
{
    /// Why each suppressed finding was suppressed, keyed by the `rule`/`subject` identity
    /// every policy in this crate already matches by.
    ///
    /// Recorded by the run that decided it rather than re-derived by a reader, for the reason
    /// [`crate::SuppressionReason`]'s own documentation gives: a later evaluation would answer
    /// with today's policy about an earlier run. Exactly one entry per member of
    /// `suppressed_findings`, in both directions.
    pub suppression_reasons: std::collections::BTreeMap<(nomos_contracts::RuleId, nomos_contracts::SubjectId), crate::SuppressionReason>,
    /// Exactly the findings for which `Finding::Can_Fail_A_Build` is true and no
    /// calibration, `Suppression` or baseline debt matched. Empty whenever `disposition` is
    /// not [`super::GateRunOutcome::Failed`].
    pub blocking_findings: Vec<Finding>,
    /// Findings for which `Finding::Can_Fail_A_Build` is true but an `AdoptionPolicy`
    /// calibration matched their rule, so they could not fail the build -- carried rather
    /// than dropped, the same reason `suppressed_findings` and `baselined_findings` both
    /// are. Checked before `suppressed_findings` and `baselined_findings`, since calibration
    /// is a coarser, rule-wide override; a finding matched by more than one reports as
    /// calibrated, not counted twice.
    pub calibrated_findings: Vec<Finding>,
    /// Findings for which `Finding::Can_Fail_A_Build` is true, no calibration matched, but a
    /// `Suppression` matched, so they could not fail the build -- carried rather than
    /// dropped, because a suppressed finding that disappears from the answer is
    /// indistinguishable from one that was never found, and the corpus's own `SUP-EVID-*`
    /// requirement is that a suppressed state stays visible rather than collapsing into
    /// silence.
    pub suppressed_findings: Vec<Finding>,
    /// Findings for which `Finding::Can_Fail_A_Build` is true, no calibration or
    /// `Suppression` matched, but a `BaselineDebt` did *and* its scope stayed within the
    /// quantity that entry accepted, so they could not fail the build either -- carried for
    /// the same reason `suppressed_findings` is, and disjoint from it and from
    /// `calibrated_findings`: a finding matched by more than one is reported once, under the
    /// earliest of calibration, suppression, then baseline.
    pub baselined_findings: Vec<Finding>,
    /// Findings a `BaselineDebt` matched whose scope holds **more** occurrences than that
    /// entry accepted.
    ///
    /// The whole group, never a subset. `OD-GATE-030` refuses attribution inside an exceeded
    /// population: a run that observes five where one was accepted knows the population is
    /// over by four and does not know *which* four, so there is no honest way to leave one of
    /// them in `baselined_findings` and move the rest here. Splitting by any stable key would
    /// invent the continuity evidence the record says nobody has, and would hand a
    /// reformatting commit the power to change which occurrences a repository is said to have
    /// adopted.
    ///
    /// These block. They are not tolerated and they are not `blocking_findings` either: a
    /// reader needs to know the reason is a tolerance that ran out of room rather than a rule
    /// nobody had addressed, and `baseline_populations` carries the arithmetic that says so.
    pub baseline_exceeded_findings: Vec<Finding>,
    /// One entry per `rule`/`subject` scope a `BaselineDebt` matched, in a stable order.
    ///
    /// Reported for every matched scope and not only the exceeded ones, because "one accepted,
    /// one observed" is the answer to *is this still within what we adopted* and a reader who
    /// only ever sees the failures cannot tell a healthy baseline from an absent one.
    pub baseline_populations: Vec<BaselinePopulation>,
}

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
