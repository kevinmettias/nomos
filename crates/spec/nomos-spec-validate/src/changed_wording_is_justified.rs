//! Wording that changed says why.

use crate::offending::Offending;
use crate::violation::Violation;
use crate::rule_outcome::RuleOutcome;
use crate::rule::Rule;
use nomos_spec_store::SpecificationStore;
/// Statements that claim to supersede a hash with no history event saying they changed.
const UNJUSTIFIED: &str = "SELECT s.statement_id
     FROM normative_statements s
     WHERE s.supersedes_hash IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM node_history h
           WHERE h.node_uid = s.node_uid AND h.event = 'content_changed'
       )
     ORDER BY s.statement_id";

/// A changed `canonical_hash` needs a recorded reason and the hash it supersedes.
pub struct ChangedWordingIsJustified;

impl Rule for ChangedWordingIsJustified
{
    fn Id(&self) -> &'static str
    {
        return "NSV-PRESERVE-003";
    }

    fn Describe(&self) -> &'static str
    {
        return "a changed canonical hash carries a supersession and a reason";
    }

    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome
    {
        return Offending(store, "normative_statements", UNJUSTIFIED, |row| {
            let id: String = row.get(0)?;
            return Ok(Violation {
                subject: id,
                detail: "supersedes a previous hash with no content_changed history event"
                    .to_owned(),
            });
        });
    }
}
