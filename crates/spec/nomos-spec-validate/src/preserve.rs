use crate::offending::{Traced, Undisposed, Undisposed_Statement};
use nomos_spec_store::Table;
use crate::rule::EveryStatementTracesToSource;
use crate::rule::ChangedWordingIsJustified;
use crate::rule::EveryBlockHasADisposition;
use crate::Rule;
use crate::RuleOutcome;
use nomos_spec_store::SpecificationStore;

pub(crate) struct EveryHeadingHasADisposition;

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
                table: Table::SourceHeadings,
                offenders: Undisposed_Statement!(
                    "source_headings",
                    "source_heading_uid",
                    "source_heading_uid"
                ),
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
