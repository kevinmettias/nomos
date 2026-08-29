//! What every preservation rule can be asked.

// The three declared rules and the two shapes a rule answers in. A rule implementation is
// only ever reached through this trait, so it belongs beneath it rather than beside it.
mod changed_wording_is_justified;
mod every_block_has_a_disposition;
mod every_statement_traces_to_source;
mod no_undeclared_filler_template;
mod rule_outcome;
mod rule_result;

pub(crate) use changed_wording_is_justified::ChangedWordingIsJustified;
pub(crate) use every_block_has_a_disposition::EveryBlockHasADisposition;
pub(crate) use every_statement_traces_to_source::EveryStatementTracesToSource;
pub(crate) use no_undeclared_filler_template::NoUndeclaredFillerTemplate;
pub use rule_outcome::RuleOutcome;
pub use rule_result::RuleResult;
use nomos_spec_store::SpecificationStore;
pub trait Rule
{
    fn Id(&self) -> &'static str;
    fn Describe(&self) -> &'static str;
    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome;
}
