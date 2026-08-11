//! Every block carried into v15 says what became of it.

use crate::offending::{Traced, Undisposed, Undisposed_Statement};
use crate::rule_outcome::RuleOutcome;
use crate::rule::Rule;
use nomos_spec_store::{SpecificationStore, Table};
/// The rule that would have caught v15.0's 282 dropped table rows — at the granularity
/// v14 recorded, which is the block. A row lost from inside a preserved table block does
/// not violate this rule, because v14's segmenter never saw rows as blocks.
pub(crate) struct EveryBlockHasADisposition;

impl Rule for EveryBlockHasADisposition
{
    fn Id(&self) -> &'static str
    {
        return "NSV-PRESERVE-002";
    }

    fn Describe(&self) -> &'static str
    {
        return "no source block is silently dropped";
    }

    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome
    {
        return Undisposed(
            store,
            &Traced {
                table: Table::SourceBlocks,
                offenders: Undisposed_Statement!(
                    "source_blocks",
                    "source_block_uid",
                    "source_block_uid"
                ),
                label: "block",
            },
        );
    }
}
