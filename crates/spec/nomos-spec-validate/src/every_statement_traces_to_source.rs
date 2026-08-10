//! Every normative statement names where it came from.

use crate::offending::Offending;
use crate::violation::Violation;
use crate::rule_outcome::RuleOutcome;
use crate::rule::Rule;
use nomos_spec_store::SpecificationStore;
/// Statements with no preserved lineage row reaching any source block.
///
/// The disposition has to be one of the two preserving ones. A statement whose only lineage
/// row says it was rewritten does not trace to the text it came from.
const UNTRACED: &str = "SELECT s.statement_id
     FROM normative_statements s
     WHERE NOT EXISTS (
         SELECT 1 FROM lineage l
         WHERE l.target_statement = s.uid
           AND l.source_block_uid IS NOT NULL
           AND l.disposition IN ('preserved-verbatim', 'preserved-normalized')
     )
     ORDER BY s.statement_id";

/// Every statement traces back to a source block that was preserved.
///
/// v15.0 shipped exactly this violation for all 363 requirements: the statements existed
/// and nothing connected them to the text they came from.
pub struct EveryStatementTracesToSource;

impl Rule for EveryStatementTracesToSource
{
    fn Id(&self) -> &'static str
    {
        return "NSV-PRESERVE-006";
    }

    fn Describe(&self) -> &'static str
    {
        return "every normative statement traces to a preserved source block";
    }

    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome
    {
        return Offending(store, "normative_statements", UNTRACED, |row| {
            let id: String = row.get(0)?;
            return Ok(Violation {
                subject: id,
                detail: "no preserved lineage row to any source block".to_owned(),
            });
        });
    }
}
