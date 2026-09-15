//! [`GateFindings`], the finding groups [`super::gate_run_response::GateRunResponse`] carries.

use super::baseline_population_response::BaselinePopulationResponse;
use nomos_contracts::Finding;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::GateFindings`].
///
/// A twin rather than a re-export because the type it mirrors does not derive `Serialize`,
/// for the reason `crate::response`'s own doc gives.
#[derive(Debug, Serialize)]
pub struct GateFindings
{
    /// Exactly the findings that failed this build. Empty whenever the run's disposition is
    /// not [`super::Disposition::Failed`].
    pub blocking_findings: Vec<Finding>,
    /// Findings an `AdoptionPolicy` calibration kept from blocking.
    pub calibrated_findings: Vec<Finding>,
    /// Findings a `Suppression` kept from blocking.
    pub suppressed_findings: Vec<Finding>,
    /// Findings a `BaselineDebt` kept from blocking, its scope having stayed within the
    /// quantity that entry accepted.
    pub baselined_findings: Vec<Finding>,
    /// Findings a `BaselineDebt` matched whose scope held more occurrences than it accepted.
    ///
    /// The whole affected group, never a subset: `OD-GATE-030` refuses to name which
    /// occurrences inside an exceeded scope are the adopted ones, because nothing a run can see
    /// answers that. These block.
    pub baseline_exceeded_findings: Vec<Finding>,
    /// One entry per baselined scope, with what it accepted and what this run found.
    pub baseline_populations: Vec<BaselinePopulationResponse>,
}

impl GateFindings
{
    pub(crate) fn From(findings: nomos_gate_orchestration::GateFindings) -> Self
    {
        return Self {
            blocking_findings: findings.blocking_findings,
            calibrated_findings: findings.calibrated_findings,
            suppressed_findings: findings.suppressed_findings,
            baselined_findings: findings.baselined_findings,
            baseline_exceeded_findings: findings.baseline_exceeded_findings,
            baseline_populations: findings.baseline_populations.into_iter().map(BaselinePopulationResponse::From).collect(),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Preserve_Each_Findings_Group_By_Name()
    {
        let findings = nomos_gate_orchestration::GateFindings {
            blocking_findings: vec![],
            calibrated_findings: vec![],
            suppressed_findings: vec![],
            baselined_findings: vec![],
            baseline_exceeded_findings: vec![],
            baseline_populations: vec![],
            suppression_reasons: Default::default(),
        };

        let response = GateFindings::From(findings);

        assert!(response.blocking_findings.is_empty());
        assert!(response.calibrated_findings.is_empty());
        assert!(response.suppressed_findings.is_empty());
        assert!(response.baselined_findings.is_empty());
        assert!(response.baseline_exceeded_findings.is_empty());
        assert!(response.baseline_populations.is_empty());
    }
}
