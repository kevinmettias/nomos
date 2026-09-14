//! [`GateFindings`], the finding groups [`super::GateRunResult`] carries.

use nomos_contracts::Finding;

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
    /// `Suppression` matched, but a `BaselineDebt` did, so they could not fail the build
    /// either -- carried for the same reason `suppressed_findings` is, and disjoint from it
    /// and from `calibrated_findings`: a finding matched by more than one is reported once,
    /// under the earliest of calibration, suppression, then baseline.
    pub baselined_findings: Vec<Finding>,
}
