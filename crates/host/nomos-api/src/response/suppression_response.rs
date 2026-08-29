//! [`SuppressionResponse`], one of [`super::gate_explain_response::GateExplainResponse::Found`]'s
//! disposition fields.

use nomos_contracts::{RuleId, SubjectId};
use nomos_gate_orchestration::Suppression;
use serde::Serialize;

use super::SuppressionDispositionResponse;

/// A serializable twin of [`nomos_gate_orchestration::Suppression`], for the same reason
/// `crate::response::disposition::Disposition` twins `GateRunOutcome`.
#[derive(Debug, Serialize)]
pub struct SuppressionResponse
{
    /// The rule this disposition applies to.
    pub rule: RuleId,
    /// The subject this disposition applies to.
    pub subject: SubjectId,
    /// Which of `SUP-*`'s six dispositions this is.
    pub disposition: SuppressionDispositionResponse,
    /// Why.
    pub rationale: String,
    /// Who is accountable for this disposition.
    pub owner: String,
}

impl SuppressionResponse
{
    pub(crate) fn From(suppression: Suppression) -> Self
    {
        return Self {
            rule: suppression.rule,
            subject: suppression.subject,
            disposition: SuppressionDispositionResponse::From(suppression.disposition),
            rationale: suppression.rationale,
            owner: suppression.owner,
        };
    }
}
