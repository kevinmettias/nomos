//! Which rules' findings count toward a `nomos gate run`'s disposition.

use nomos_contracts::RuleId;

/// Which rules' findings a run's disposition is reduced from.
///
/// Deliberately a post-hoc filter over [`nomos_contracts::Finding::rule`], not a change to
/// what [`nomos_check_orchestration::Run`] executes: that function calls every rule
/// unconditionally today, and giving it a per-call subset would mean changing its own
/// signature across every caller (`nomos-cli`'s `check` and `gate`, `nomos-gate-
/// orchestration::Run_Gate`, `vacuity.rs`) -- a separate, larger item, not this one. A
/// selected-out rule's findings are still computed and still appear in
/// [`crate::GateRunResult::check_outcome`] in full; they are excluded only from
/// [`crate::GateRunResult::blocking_findings`] and the [`crate::GateRunOutcome`] reduced
/// from them. That is real selection of what can fail a build, honestly short of real
/// selection of what runs.
///
/// An empty `include` is "select every rule," the state every existing caller is in today:
/// `Default` gives that state, so CI's `gate run --root .` is unchanged in behavior by this
/// type existing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuleSelector
{
    /// The rules whose findings count, or every rule's do when this is empty.
    pub include: Vec<RuleId>,
}

impl RuleSelector
{
    /// Whether `rule`'s findings count toward this run's disposition.
    #[must_use]
    pub fn Matches(&self, rule: &RuleId) -> bool
    {
        return self.include.is_empty() || self.include.contains(rule);
    }
}

#[cfg(test)]
mod tests
{
    use super::RuleSelector;
    use nomos_contracts::RuleId;

    #[test]
    fn Test_An_Empty_Selector_Should_Match_Every_Rule()
    {
        let selector = RuleSelector::default();

        assert!(selector.Matches(&RuleId::New("naming-convention")));
    }

    #[test]
    fn Test_A_Named_Selector_Should_Admit_Only_Its_Own_Rules()
    {
        let selector = RuleSelector { include: vec![RuleId::New("naming-convention")] };

        assert!(selector.Matches(&RuleId::New("naming-convention")));
        assert!(!selector.Matches(&RuleId::New("dependency-direction")));
    }
}
