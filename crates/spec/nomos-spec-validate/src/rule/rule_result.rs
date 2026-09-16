//! One rule, and what it answered.

use crate::RuleOutcome;
#[derive(Clone, Debug)]
pub struct RuleResult
{
    pub id: String,
    pub outcome: RuleOutcome,
}
