//! One disposition over one rule's finding on one subject.

use super::SuppressionDisposition;
use nomos_contracts::{Finding, RuleId, SubjectId};

/// One disposition over one rule's finding on one subject.
///
/// `owner`/`approver`/dates/revalidation triggers are `SUP-*`'s other required fields and
/// are deliberately not here yet -- this increment's own crate doc says why: no caller
/// constructs a `Suppression` at all today, so enforcing fields nothing populates would be
/// validating against nothing. Adding them is a later increment once something authors one
/// for real.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suppression
{
    /// The rule this disposition applies to.
    pub rule: RuleId,
    /// The subject this disposition applies to.
    pub subject: SubjectId,
    /// Which of `SUP-*`'s six dispositions this is.
    pub disposition: SuppressionDisposition,
    /// Why -- required, because a disposition with no stated reason is not distinguishable
    /// from silence.
    pub rationale: String,
    /// Who is accountable for this disposition.
    pub owner: String,
}

impl Suppression
{
    /// Whether this disposition applies to `finding` -- the same `rule`/`subject` identity
    /// `Finding` already carries, so matching invents no addressing scheme of its own.
    #[must_use]
    pub fn Is_Applicable_To(&self, finding: &Finding) -> bool
    {
        return self.rule == finding.rule && self.subject == finding.subject;
    }
}
