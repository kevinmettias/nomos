//! A previewed budget for one rule at one check-or-fix stage.

use core::time::Duration;

use nomos_contracts::RuleId;
use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-011`: "Users shall be able to preview estimated token, latency,
/// resource, and monetary budgets by rule/check and by check-versus-fix stage."
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CheckOrFixStage
{
    Check,
    Fix,
}

/// `MODEL-ROUTE-011`: "Estimates shall expose assumptions and shall not be
/// represented as guaranteed provider charges or quality outcomes."
///
/// Four named budget dimensions, grouped by rule and by [`CheckOrFixStage`] -- both
/// real or directly-buildable identities -- plus the required `assumptions` field the
/// second sentence names. `monetary` stays a raw string placeholder rather than a
/// typed currency amount: a real typed shape for money is a separate, unbuilt
/// requirement's territory. The "shall not be represented as guaranteed" clause is a
/// presentation constraint on how a client renders this struct, not an additional
/// field, and is not encoded here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetEstimate
{
    pub rule: RuleId,
    pub stage: CheckOrFixStage,
    pub token: Option<u64>,
    pub latency: Option<Duration>,
    pub resource: Option<String>,
    pub monetary: Option<String>,
    /// "Estimates shall expose assumptions" -- required, not optional.
    pub assumptions: Vec<String>,
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_An_Estimate_Carries_Exactly_What_It_Was_Given()
    {
        let estimate = BudgetEstimate {
            rule: RuleId::New("check-naming-convention"),
            stage: CheckOrFixStage::Fix,
            token: Some(1_200),
            latency: Some(Duration::from_secs(4)),
            resource: None,
            monetary: Some("~$0.02 at current list pricing".to_owned()),
            assumptions: vec!["pricing reflects the provider's public list rate, not a negotiated one".to_owned()],
        };

        assert_eq!(estimate.stage, CheckOrFixStage::Fix);
        assert_eq!(estimate.token, Some(1_200));
        assert!(!estimate.assumptions.is_empty());
    }
}
