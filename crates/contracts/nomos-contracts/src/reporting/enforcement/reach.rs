use serde::{Deserialize, Serialize};

use super::{EnforcementBreach, EnforcerRef, GateCategory};
use crate::RuleId;

/// What a rule claims about its enforcement, and what is actually true.
///
/// Three separate fields, and the separation is the whole design. A rule names its
/// enforcers (`declared`), states what it believes that amounts to (`expected`), and the
/// wiring walk says what it really amounts to (`computed`). Collapsing any two of these
/// destroys the property being protected:
///
/// - Dropping `expected` and inferring it from `declared` is what the first draft of
///   this type did, and it made a rule naming a check that nothing invokes read as
///   truthful — because it did name a check. Naming an enforcer is not a claim that the
///   enforcer runs; those are the two halves this whole module exists to separate.
/// - Overwriting `expected` with `computed` erases the disagreement, and the
///   disagreement *is* the finding.
///
/// A rule may honestly declare [`GateCategory::Unreachable`]. That is a true statement
/// about a real gap, and it is exactly what makes the gap countable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reach
{
    /// The rule this describes.
    pub rule: RuleId,
    /// The enforcers the rule names.
    pub declared: Vec<EnforcerRef>,
    /// What the rule claims those enforcers amount to. Authored.
    pub expected: GateCategory,
    /// What those enforcers actually amount to, derived from the real wiring.
    pub computed: GateCategory,
    /// Every way the declaration is false.
    pub breaches: Vec<EnforcementBreach>,
}

impl Reach
{
    /// Whether the rule's claims about its own enforcement are true.
    ///
    /// Three conditions, all necessary:
    ///
    /// 1. No breach was found — every named enforcer resolves, applies, is in this
    ///    build, and reaches what the rule binds.
    /// 2. The claimed gate matches the computed one.
    /// 3. Naming only `review` and claiming [`GateCategory::Review`] agree with each
    ///    other. A rule that names a real check while claiming `review` is understating
    ///    working enforcement; a rule that names only `review` while claiming to block
    ///    is claiming a machine that does not exist.
    #[must_use]
    pub fn Is_Truthful(&self) -> bool
    {
        if !self.breaches.is_empty()
        {
            return false;
        }
        if self.expected != self.computed
        {
            return false;
        }

        let declares_only_review = self
            .declared
            .iter()
            .all(|enforcer| matches!(enforcer, EnforcerRef::Review));

        return declares_only_review == (self.expected == GateCategory::Review);
    }

    /// Whether this rule is mechanically enforced in a way that can fail a build.
    ///
    /// Uses `computed`, never `expected`. What a rule claims about itself has no
    /// bearing on whether a violation actually stops anything.
    #[must_use]
    pub fn Is_Enforced(&self) -> bool
    {
        return self.breaches.is_empty() && self.computed.Can_Fail_A_Build();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Is_Truthful_Should_Accept_A_Review_Declaration_That_Nothing_Enforces()
    {
        assert!(
            Reach_Fixture(
                vec![EnforcerRef::Review],
                GateCategory::Review,
                GateCategory::Review
            )
            .Is_Truthful()
        );
    }

    /// The headline case, and the one the first draft of this type got wrong. A rule
    /// that names a check while claiming to block, when nothing invokes that check, is
    /// the state both prototypes accumulated. It must not read as truthful merely
    /// because it named something real.
    #[test]
    fn Test_Declaring_A_Check_That_Nothing_Invokes_Should_Not_Be_Truthful()
    {
        let overclaimed = Reach_Fixture(
            vec![Named_Check("check-strategy-docs")],
            GateCategory::Blocking,
            GateCategory::Unreachable,
        );

        assert!(!overclaimed.Is_Truthful());
        assert!(!overclaimed.Is_Enforced());
    }

    /// The counterpart, and the reason `expected` exists as a separate field: a rule
    /// may *honestly* declare that its enforcer is unreachable. That is a true
    /// statement about a real gap, it is what makes the gap countable, and it must not
    /// be conflated with the overclaim above.
    #[test]
    fn Test_Honestly_Declared_Unreachable_Should_Be_Truthful()
    {
        let honest = Reach_Fixture(
            vec![Named_Check("check-strategy-docs")],
            GateCategory::Unreachable,
            GateCategory::Unreachable,
        );

        assert!(honest.Is_Truthful(), "an admitted gap is an honest declaration");
        assert!(!honest.Is_Enforced(), "but it is still not enforcement");
    }

    /// The inverse understatement: claiming review while a gate really does enforce it.
    /// Less dangerous, still false, and it hides working enforcement from the roll-up.
    #[test]
    fn Test_Declaring_Review_While_A_Gate_Enforces_It_Should_Not_Be_Truthful()
    {
        assert!(
            !Reach_Fixture(
                vec![EnforcerRef::Review],
                GateCategory::Blocking,
                GateCategory::Blocking
            )
            .Is_Truthful()
        );
    }

    /// Naming a real, running check while claiming only `review` understates it. The
    /// gate categories agree here, so only the declared/expected consistency rule
    /// catches this one.
    #[test]
    fn Test_Naming_A_Check_While_Claiming_Review_Should_Not_Be_Truthful()
    {
        assert!(
            !Reach_Fixture(
                vec![Named_Check("check-orphan-modules")],
                GateCategory::Review,
                GateCategory::Review
            )
            .Is_Truthful()
        );
    }

    #[test]
    fn Test_Is_Truthful_And_Is_Enforced_Should_Both_Be_Refused_By_A_Breach_Even_When_Blocking()
    {
        let mut reach = Reach_Fixture(
            vec![Named_Check("check-cohesion")],
            GateCategory::Blocking,
            GateCategory::Blocking,
        );
        reach.breaches.push(EnforcementBreach::OutOfReach {
            enforcer: Named_Check("check-cohesion"),
            unreached: vec!["kotlin".to_owned(), "swift".to_owned()],
        });

        assert!(!reach.Is_Truthful());
        assert!(
            !reach.Is_Enforced(),
            "a check that cannot see most of what the rule binds is not enforcement"
        );
    }

    fn Named_Check(name: &str) -> EnforcerRef
    {
        return EnforcerRef::Check {
            name: name.to_owned(),
        };
    }

    fn Reach_Fixture(
        declared: Vec<EnforcerRef>,
        expected: GateCategory,
        computed: GateCategory,
    ) -> Reach
    {
        return Reach {
            rule: RuleId::New("example-rule"),
            declared,
            expected,
            computed,
            breaches: Vec::new(),
        };
    }
}
