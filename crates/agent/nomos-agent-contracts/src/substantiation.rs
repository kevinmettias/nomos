//! Which portions of a work result its producing executor could substantiate.

use crate::PortionSubstantiation;

/// Which portions of a [`WorkResult`](crate::WorkResult) the producing executor could
/// substantiate.
///
/// One named entry per portion, so a reader asks the value directly — `result.substantiation.plan`
/// — and no portion can be silently omitted from the declaration. The six entries are the six
/// fields of `WorkResult`, so a seventh field added there is a compile error here until its
/// entry is declared. That is the property this type exists for: it is what makes an
/// incomplete declaration impossible rather than merely discouraged.
///
/// There is deliberately no `Default`. A default would declare every portion the same way
/// without anyone deciding it, which is the failure this type replaces — a value that looks
/// declared while saying nothing.
///
/// The rule a consumer applies, stated here so it is derived rather than inferred: an entry
/// that is [`PortionSubstantiation::Substantiated`] whose value is empty means the task
/// produced none, and an entry that is [`PortionSubstantiation::Unsubstantiated`] means the
/// value says nothing about the task whatever it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Substantiation
{
    /// What the producer could say about the proposed plan.
    pub plan: PortionSubstantiation,
    /// What the producer could say about the findings it claims.
    pub claims: PortionSubstantiation,
    /// What the producer could say about the individual verifications it ran or proposes.
    pub tests: PortionSubstantiation,
    /// What the producer could say about the single predicate it asks be run.
    pub requested_verification: PortionSubstantiation,
    /// What the producer could say about the assumptions it states.
    pub assumptions: PortionSubstantiation,
    /// What the producer could say about the questions it leaves open.
    pub unresolved_questions: PortionSubstantiation,
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::UnsubstantiatedReason;

    /// The declaration a judgment-only dispatch publishes: it grounded the two portions it
    /// read from the answer, and no other.
    fn Judgment_Only() -> Substantiation
    {
        let ungrounded =
            PortionSubstantiation::Unsubstantiated(UnsubstantiatedReason::ProducerCannotGround);

        return Substantiation {
            plan: ungrounded.clone(),
            claims: ungrounded.clone(),
            tests: ungrounded.clone(),
            requested_verification: ungrounded,
            assumptions: PortionSubstantiation::Substantiated,
            unresolved_questions: PortionSubstantiation::Substantiated,
        };
    }

    #[test]
    fn Test_A_Judgment_Only_Declaration_Should_Substantiate_Exactly_Two_Portions()
    {
        let declaration = Judgment_Only();

        assert!(declaration.assumptions.Is_Substantiated());
        assert!(declaration.unresolved_questions.Is_Substantiated());
        assert!(!declaration.plan.Is_Substantiated());
        assert!(!declaration.claims.Is_Substantiated());
        assert!(!declaration.tests.Is_Substantiated());
        assert!(!declaration.requested_verification.Is_Substantiated());
    }

    /// The distinction the type exists for: two portions holding the same empty value are
    /// told apart by their declarations, which a boolean or a bare absence could not do.
    #[test]
    fn Test_Two_Empty_Portions_Should_Be_Distinguishable_By_Their_Declarations()
    {
        let declaration = Judgment_Only();

        let empty_and_grounded = declaration.assumptions.Is_Substantiated();
        let empty_and_ungrounded = declaration.claims.Is_Substantiated();

        assert!(empty_and_grounded);
        assert!(!empty_and_ungrounded);
    }
}
