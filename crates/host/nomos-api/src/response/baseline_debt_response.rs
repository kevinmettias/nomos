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
    /// The path this entry was written with, or `null` when no file declared it.
    ///
    /// Display material beside `subject`'s identity, for the reason
    /// [`super::baseline_population_response::BaselinePopulationResponse::declared_path`] gives
    /// at length: it is what a caller can name the entry by to the person who wrote it.
    ///
    /// `explain` carries this because it answers *why* one finding is tolerated, and the
    /// answer is only actionable if the reader can find the line that tolerates it.
    pub declared_path: Option<String>,
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
        return Self {
            rule: debt.rule,
            subject: debt.subject,
            declared_path: debt.declared_path,
            rationale: debt.rationale,
            allowance: BaselineAllowanceResponse::From(debt.allowance),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_rules::COMPLETENESS_MIRROR;

    /// What the debt accepts, named once so the constructor below and the assertion
    /// about its rendering cannot drift apart.
    const ACCEPTED_OCCURRENCES: u32 = 2;

    #[test]
    fn Test_From_Should_Carry_The_Debts_Rule_Subject_Rationale_And_Allowance()
    {
        let debt = Debt_Accepting(nomos_gate_orchestration::BaselineAllowance::AtMost(ACCEPTED_OCCURRENCES));
        let response = BaselineDebtResponse::From(debt.clone());

        assert_eq!(response.rule, debt.rule);
        assert_eq!(response.subject, debt.subject);
        assert_eq!(response.rationale, debt.rationale);

        let rendered = serde_json::to_value(&response)
            .expect("every field is a plain id, string or optional string, so the derive cannot fail");
        assert_eq!(rendered.pointer("/allowance/accepted_occurrence_count").and_then(serde_json::Value::as_u64), Some(u64::from(ACCEPTED_OCCURRENCES)), "{rendered}");
        assert_eq!(rendered.get("declared_path").and_then(serde_json::Value::as_str), Some("./a.rs"), "the spelling the author wrote is what makes this entry findable again: {rendered}");
    }

    fn Debt_Accepting(allowance: nomos_gate_orchestration::BaselineAllowance) -> BaselineDebt
    {
        return BaselineDebt {
            rule: RuleId::New(COMPLETENESS_MIRROR),
            subject: nomos_model::Subject_Of_Path("a.rs"),
            rationale: "a real rationale".to_owned(),
            allowance,
            declared_path: Some("./a.rs".to_owned()),
        };
    }
}
