//! One rule, and what it answered.

use crate::RuleOutcome;
#[derive(Clone, Debug)]
pub struct Result
{
    pub id: String,
    pub outcome: RuleOutcome,
}
