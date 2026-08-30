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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_gate_orchestration::SuppressionDisposition;
    use nomos_rules::COMPLETENESS_MIRROR;

    #[test]
    fn Test_From_Should_Carry_The_Suppressions_Rule_Subject_Disposition_Rationale_And_Owner()
    {
        let suppression = Suppression {
            rule: RuleId::New(COMPLETENESS_MIRROR),
            subject: nomos_model::Subject_Of_Path("a.rs"),
            disposition: SuppressionDisposition::FormalRiskAcceptance,
            rationale: "a real rationale".to_owned(),
            owner: "a real owner".to_owned(),
        };

        let response = SuppressionResponse::From(suppression.clone());

        assert_eq!(response.rule, suppression.rule);
        assert_eq!(response.subject, suppression.subject);
        assert_eq!(response.disposition, SuppressionDispositionResponse::From(suppression.disposition));
        assert_eq!(response.rationale, suppression.rationale);
        assert_eq!(response.owner, suppression.owner);
    }
}
