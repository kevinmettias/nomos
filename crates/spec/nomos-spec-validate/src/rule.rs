//! What every preservation rule can be asked.

use crate::rule_outcome::RuleOutcome;
use nomos_spec_store::SpecificationStore;
pub trait Rule
{
    fn Id(&self) -> &'static str;
    fn Describe(&self) -> &'static str;
    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome;
}
