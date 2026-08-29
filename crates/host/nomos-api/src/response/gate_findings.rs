//! [`GateFindings`], the finding groups [`super::gate_run_response::GateRunResponse`] carries.

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
    /// Findings a `BaselineDebt` kept from blocking.
    pub baselined_findings: Vec<Finding>,
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
        };
    }
}
