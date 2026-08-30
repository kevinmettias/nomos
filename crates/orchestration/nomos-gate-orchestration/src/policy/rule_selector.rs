//! Which rules' findings count toward a `nomos gate run`'s disposition.

use nomos_contracts::RuleId;

/// Which rules' findings a run's disposition is reduced from.
///
/// `include` is also, since `OD-GATE-017`, the same selection [`crate::gate_environment::Judged_Sources`]
/// passes through to [`nomos_check_orchestration::Run`] itself: a selected-out rule's own
/// materialization does not run at all, and its finding never exists to appear in
/// [`crate::GateRunResult::check_outcome`] or anywhere else. What this type still does, on
/// top of that, is what its name says -- filtering which of the rules that *did* run counts
/// toward [`crate::GateRunResult::blocking_findings`] and the [`crate::GateRunOutcome`]
/// reduced from them, the layer [`crate::finding_query::Explain_Gate`] deliberately bypasses by
/// passing `Run` an empty selection regardless of this type's own value.
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
    pub fn Is_Included(&self, rule: &RuleId) -> bool
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

        assert!(selector.Is_Included(&RuleId::New("naming-convention")));
    }

    #[test]
    fn Test_Is_Included_Should_Admit_Only_Its_Own_Rules()
    {
        let selector = RuleSelector { include: vec![RuleId::New("naming-convention")] };

        assert!(selector.Is_Included(&RuleId::New("naming-convention")));
        assert!(!selector.Is_Included(&RuleId::New("dependency-direction")));
    }
}
