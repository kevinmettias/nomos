//! [`UndeclaredValues`], every value a run's rules read that the repository never declared.
//!
//! `OD-RULES-011` version 3's decisions 1 and 3. A rule judged against a value this workspace
//! substituted, or judged nothing for want of a norm, reaches the same findings as a rule holding
//! a repository to what it declared, and only this says which. It is reported beside the claim and
//! the population and apart from both. [`crate::Claim_Of`] reads findings alone, so a repository
//! that declares every value reaches the same claim it would reach declaring none, and nothing a
//! gate derives from the claim moves. It carries no `GateCategory` and is not a finding: a value is
//! not a subject.
//!
//! The root records what each rule's descriptor answers and interprets no key: which value a rule
//! read, and whether the repository declared it, are `nomos-rules`' to say.

use nomos_contracts::RuleId;
use nomos_rules::UndeclaredValue;

/// For each rule a run selected and judged over a nonempty population, in the order the run judged
/// them, every value it read that the repository did not declare -- a rule with none is not listed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UndeclaredValues
{
    named: Vec<(RuleId, Vec<UndeclaredValue>)>,
}

impl UndeclaredValues
{
    /// No rule named anything yet.
    #[must_use]
    pub fn New() -> Self
    {
        return Self::default();
    }

    /// Records the values `rule` read that the repository did not declare, and nothing for a rule
    /// that read none.
    pub fn Note(&mut self, rule: RuleId, values: Vec<UndeclaredValue>)
    {
        if values.is_empty()
        {
            return;
        }

        self.named.push((rule, values));
    }

    /// The values `rule` read that the repository did not declare, empty when it read none or the
    /// run did not judge it.
    #[must_use]
    pub fn Of(&self, rule: &RuleId) -> &[UndeclaredValue]
    {
        return self.named.iter().find(|(named, _)| return named == rule).map_or(&[], |(_, values)| return values.as_slice());
    }

    /// Every rule that read a value the repository did not declare, with those values, in the order
    /// the run judged them.
    #[must_use]
    pub fn Named(&self) -> &[(RuleId, Vec<UndeclaredValue>)]
    {
        return &self.named;
    }

    /// Whether no rule read a value the repository did not declare.
    #[must_use]
    pub fn Is_Empty(&self) -> bool
    {
        return self.named.is_empty();
    }
}
