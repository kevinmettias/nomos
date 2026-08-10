//! One rule, and what it answered.

use crate::rule_outcome::RuleOutcome;
#[derive(Clone, Debug)]
pub struct RuleResult
{
    pub id: String,
    pub outcome: RuleOutcome,
}
