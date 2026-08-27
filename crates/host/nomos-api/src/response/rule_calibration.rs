//! [`RuleCalibrationResponse`], one of [`super::explain::GateExplainResponse::Found`]'s
//! disposition fields.

use nomos_contracts::RuleId;
use nomos_gate_orchestration::RuleCalibration;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::RuleCalibration`], for the same reason
/// `crate::response::disposition::Disposition` twins `GateRunOutcome`.
#[derive(Debug, Serialize)]
pub struct RuleCalibrationResponse
{
    /// The rule this calibration applies to.
    pub rule: RuleId,
    /// Why this rule is not yet blocking for this repository.
    pub rationale: String,
}

impl RuleCalibrationResponse
{
    pub(crate) fn From(calibration: RuleCalibration) -> Self
    {
        return Self { rule: calibration.rule, rationale: calibration.rationale };
    }
}
