//! [`RuleCalibrationResponse`], one of [`super::gate_explain_response::GateExplainResponse::Found`]'s
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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_rules::COMPLETENESS_MIRROR;

    #[test]
    fn Test_From_Should_Carry_The_Calibrations_Rule_And_Rationale()
    {
        let calibration = RuleCalibration { rule: RuleId::New(COMPLETENESS_MIRROR), rationale: "a real rationale".to_owned() };

        let response = RuleCalibrationResponse::From(calibration.clone());

        assert_eq!(response.rule, calibration.rule);
        assert_eq!(response.rationale, calibration.rationale);
    }
}
