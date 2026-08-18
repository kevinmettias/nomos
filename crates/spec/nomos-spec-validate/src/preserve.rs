use crate::offending::{Traced, Undisposed, Undisposed_Statement};
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
        use nomos_spec_store::Table;

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
// Five unrelated types in one vector, and the set is meant to stay open: `DECLARED_RULES`
// closes it at run time, in both directions, so that a rule implemented and never listed here
// is a value `Validate` can be handed and report. Making the ruleset a tuple of types would
// put that disagreement beyond expressing.
pub fn Registered() -> Vec<Box<dyn Rule>>
{
    use crate::rule::EveryStatementTracesToSource;
    use crate::rule::ChangedWordingIsJustified;
    use crate::rule::EveryBlockHasADisposition;
    use crate::rule::NoUndeclaredFillerTemplate;

    return vec![
        Box::new(EveryHeadingHasADisposition),
        Box::new(EveryBlockHasADisposition),
        Box::new(ChangedWordingIsJustified),
        Box::new(EveryStatementTracesToSource),
        Box::new(NoUndeclaredFillerTemplate),
    ];
}
