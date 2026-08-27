//! [`BaselineDebtResponse`], one of [`super::explain::GateExplainResponse::Found`]'s
//! disposition fields.

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
}

impl BaselineDebtResponse
{
    pub(crate) fn From(debt: BaselineDebt) -> Self
    {
        return Self { rule: debt.rule, subject: debt.subject, rationale: debt.rationale };
    }
}
