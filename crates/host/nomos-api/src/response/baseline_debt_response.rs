//! [`BaselineDebtResponse`], one of [`super::gate_explain_response::GateExplainResponse::Found`]'s
//! disposition fields.

use super::baseline_allowance_response::BaselineAllowanceResponse;
use nomos_contracts::{RuleId, SubjectId};
use nomos_gate_orchestration::BaselineDebt;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::BaselineDebt`], for the same reason
/// `crate::response::disposition::Disposition` twins `GateRunOutcome`.
#[derive(Debug, Serialize)]
pub struct BaselineDebtResponse
{
    /// The rule this debt applies to.
    pub rule: RuleId,
    /// The subject this debt applies to.
    pub subject: SubjectId,
    /// Why this finding is tolerated rather than fixed.
    pub rationale: String,
    /// How many occurrences this entry accepted at adoption.
    ///
    /// `OD-GATE-030`: a later run may tolerate no more than this quantity. It says nothing
    /// about whether the occurrences seen now are the ones that were adopted.
    pub allowance: BaselineAllowanceResponse,
}

impl BaselineDebtResponse
{
    pub(crate) fn From(debt: BaselineDebt) -> Self
    {
        return Self { rule: debt.rule, subject: debt.subject, rationale: debt.rationale, allowance: BaselineAllowanceResponse::From(debt.allowance) };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_rules::COMPLETENESS_MIRROR;

    #[test]
    fn Test_From_Should_Carry_The_Debts_Rule_Subject_Rationale_And_Allowance()
    {
        let debt = BaselineDebt {
            rule: RuleId::New(COMPLETENESS_MIRROR),
            subject: nomos_model::Subject_Of_Path("a.rs"),
            rationale: "a real rationale".to_owned(),
            allowance: nomos_gate_orchestration::BaselineAllowance::AtMost(2),
        };

        let response = BaselineDebtResponse::From(debt.clone());

        assert_eq!(response.rule, debt.rule);
        assert_eq!(response.subject, debt.subject);
        assert_eq!(response.rationale, debt.rationale);
        let rendered = serde_json::to_value(&response).expect("always serializes");
        assert_eq!(rendered.pointer("/allowance/accepted_occurrence_count").and_then(serde_json::Value::as_u64), Some(2), "{rendered}");
    }
}
