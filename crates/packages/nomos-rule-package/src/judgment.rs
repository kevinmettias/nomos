//! Whether anything mechanically judges the rule a package declares.

/// How the rule this package declares is decided.
///
/// `OD-RULES-022` decided that a declaration states its own judgment, and that this is what
/// makes composition's resolution total: a declaration is either mechanical, in which case an
/// implementation must exist and resolution fails if it does not, or model-judged, in which
/// case there is none to resolve and the rule contributes nothing to a deterministic run.
///
/// This is not [`crate::ApplicabilitySemantics`] under another name, and the two are easy to
/// confuse. That one asks how much of its subject a judgment covers --
/// `AlwaysSupported` against `StructurallyPartial` -- and presupposes a judgment exists to
/// cover anything. This one asks whether one exists at all. A model-judged rule has no
/// applicability semantics worth stating for the same reason it has no capability
/// requirements: nothing runs.
///
/// The population this exists for is not hypothetical. The Go predecessor
/// (`github.com/kevinmettias/nomos-proto`) declares 1,143 rules, and its own
/// `kernel/rules/rulespec/rule.go` states that 728 of 1,116 are decided by a model rather
/// than by a parser. Without this distinction those rules could only be declared by
/// pretending an implementation exists, and a rule that reports clean because nothing ran is
/// the defect `OD-GATE-001` exists to prevent, installed one layer up from where that record
/// found it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Judgment
{
    /// A linked implementation decides this rule, and resolution refuses the package if none
    /// is registered for its `rule_id`.
    Mechanical,
    /// No implementation decides this rule; a model reading source does. The rule is declared,
    /// citable and projectable, and contributes nothing to a deterministic run.
    ///
    /// This is a truthful thing for a rule to be, and a different thing from a rule that ran
    /// and found nothing. `agent_guidance` is the field such a declaration carries instead of
    /// `required_capabilities`, and it is empty for every rule shipped so far precisely
    /// because no rule it was written for could be declared until now.
    ModelJudged,
}

impl Judgment
{
    /// The variant's stable, lowercase wire spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Mechanical => "mechanical",
            Self::ModelJudged => "model_judged",
        };
    }

    /// Whether a linked implementation must exist for this declaration to resolve.
    ///
    /// Named rather than left to the caller to match on, because the resolution step
    /// `OD-RULES-022` describes asks exactly this question and a `match` at every call site is
    /// how two of them come to disagree about which arm means what.
    #[must_use]
    pub const fn Needs_An_Implementation(self) -> bool
    {
        return matches!(self, Self::Mechanical);
    }
}

impl core::fmt::Display for Judgment
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::Judgment;

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        assert_ne!(Judgment::Mechanical.Label(), Judgment::ModelJudged.Label());
    }

    /// Exactly one arm demands an implementation, which is the whole of what the resolution
    /// step reads this for.
    #[test]
    fn Test_Only_A_Mechanical_Judgment_Needs_An_Implementation()
    {
        assert!(Judgment::Mechanical.Needs_An_Implementation());
        assert!(!Judgment::ModelJudged.Needs_An_Implementation());
    }
}
