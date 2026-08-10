use crate::offending::Traced;
use crate::offending::Undisposed;
use crate::every_statement_traces_to_source::EveryStatementTracesToSource;
use crate::changed_wording_is_justified::ChangedWordingIsJustified;
use crate::every_block_has_a_disposition::EveryBlockHasADisposition;
use crate::rule::Rule;
use crate::rule_outcome::RuleOutcome;
use nomos_spec_store::SpecificationStore;

pub struct EveryHeadingHasADisposition;

impl Rule for EveryHeadingHasADisposition
{
    fn Id(&self) -> &'static str
    {
        return "NSV-PRESERVE-001";
    }

    fn Describe(&self) -> &'static str
    {
        return "every source heading is preserved, superseded or explicitly omitted";
    }

    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome
    {
        return Undisposed(
            store,
            &Traced {
                table: "source_headings",
                lineage: "source_heading_uid",
                omission: "source_heading_uid",
                label: "heading",
            },
        );
    }
}



#[must_use]
pub fn Registered() -> Vec<Box<dyn Rule>>
{
    return vec![
        Box::new(EveryHeadingHasADisposition),
        Box::new(EveryBlockHasADisposition),
        Box::new(ChangedWordingIsJustified),
        Box::new(EveryStatementTracesToSource),
    ];
}
