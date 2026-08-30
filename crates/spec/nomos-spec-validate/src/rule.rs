//! What every preservation rule can be asked.

// The three declared rules and the two shapes a rule answers in. A rule implementation is
// only ever reached through this trait, so it belongs beneath it rather than beside it.
mod changed_wording_is_justified;
mod every_block_has_a_disposition;
mod every_statement_traces_to_source;
mod no_undeclared_filler_template;
mod outcome;
mod result;

pub(crate) use changed_wording_is_justified::ChangedWordingIsJustified;
pub(crate) use every_block_has_a_disposition::EveryBlockHasADisposition;
pub(crate) use every_statement_traces_to_source::EveryStatementTracesToSource;
pub(crate) use no_undeclared_filler_template::NoUndeclaredFillerTemplate;
pub use outcome::Outcome as RuleOutcome;
// Bare `Result` would shadow `std::result::Result`, which nearly every fallible function in
// this workspace returns unqualified, so the flat surface keeps the longer, collision-free
// name even though the declaration itself (`rule::result::Result`) is now the trimmed one.
pub use result::Result as RuleResult;
use nomos_spec_store::SpecificationStore;
pub trait Rule
{
    fn Id(&self) -> &'static str;
    fn Describe(&self) -> &'static str;
    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome;
}
