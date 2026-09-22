//! The lowest class of evidence a gate lets a finding block on.

use nomos_contracts::EvidenceClass;

/// The lowest [`EvidenceClass`] a finding must carry before this gate lets it fail a build.
///
/// `OD-GATE-034`: an evidence requirement is a floor a gate states over the ordering
/// [`EvidenceClass`] already publishes -- weakest first, `AgentJudged` through
/// `Authoritative` -- and not a second notion of strength. A gate that wants only mechanical
/// findings to block states [`EvidenceClass::Approximate`], because `EvidenceClass::
/// Is_Mechanical` is that point on the same order and nothing else. `EVID-001` forbids
/// reducing the classification to one confidence number and a floor does not: it names a
/// class, and the eight classes stay distinct on both sides of it.
///
/// A declared constant, never a condition. `OD-ROADMAP-003`'s surviving constraint is that a
/// gate policy family is a field [`crate::policy::Resolve_Gate_Policy`] reads off
/// `nomos-gate.json`, so nothing here consults store state, cost or a prior materialization
/// -- not "require `Verified` when the store already holds a stronger fact", not "lower the
/// floor when materializing the evidence would cost more". The moment one of those is written
/// the field has stopped being a declared fact.
///
/// `Default` gives [`Self::Unset`], which is the state every existing caller and CI's own
/// `gate run --root .` are in today -- every class may block -- for the reason
/// [`crate::CoveragePolicy`]'s own default gives: a field nobody wrote keeps the behavior it
/// had.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EvidenceFloor
{
    /// No floor. A finding blocks on its gate category and its applicability alone, whatever
    /// class of evidence backs it -- `OD-COMPLETENESS-004`'s own default, carried forward.
    #[default]
    Unset,
    /// Only a finding whose evidence is at least this class may block.
    ///
    /// Compared with `EvidenceClass`'s own derived `Ord` and with nothing else. A floor that
    /// ranked the classes its own way would be the universal confidence number `EVID-001`
    /// forbids, wearing another name.
    AtLeast(EvidenceClass),
}

impl EvidenceFloor
{
    /// Whether `evidence` is strong enough to block under this floor.
    ///
    /// Read beside `Finding::Can_Fail_A_Build`'s own two conditions rather than inside it:
    /// a `Finding` does not know which gate is reading it, so the contract type keeps
    /// deciding what *can* fail a build and the gate keeps deciding what does.
    #[must_use]
    pub fn Admits(self, evidence: EvidenceClass) -> bool
    {
        return match self
        {
            Self::Unset => true,
            Self::AtLeast(floor) => evidence >= floor,
        };
    }

    /// The class this gate requires, or `None` when it requires none.
    ///
    /// What a report cites when it names the floor that moved a finding out of the blocking
    /// bucket. [`Self::Unset`] moves nothing, so it has nothing to cite.
    #[must_use]
    pub const fn Required(self) -> Option<EvidenceClass>
    {
        return match self
        {
            Self::Unset => None,
            Self::AtLeast(floor) => Some(floor),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::EvidenceFloor;
    use nomos_contracts::EvidenceClass;

    /// Every class, weakest first, in the order `evidence_class.rs` declares them.
    ///
    /// Written out rather than read from a declared list, because `EvidenceClass` publishes
    /// none and inventing one here would be a universe this crate owes a mirror for. What
    /// keeps it honest is `Test_The_Floor_Should_Follow_EvidenceClass_Own_Ordering` below:
    /// it asserts this sequence against the `Ord` the contract type derives, so a class added
    /// or reordered there fails here rather than silently narrowing what the floor was
    /// checked over.
    const WEAKEST_FIRST: [EvidenceClass; 8] = [
        EvidenceClass::AgentJudged,
        EvidenceClass::HumanAsserted,
        EvidenceClass::Predicted,
        EvidenceClass::Approximate,
        EvidenceClass::Derived,
        EvidenceClass::Observed,
        EvidenceClass::Verified,
        EvidenceClass::Authoritative,
    ];

    #[test]
    fn Test_Default_Should_Be_Unset()
    {
        assert_eq!(EvidenceFloor::default(), EvidenceFloor::Unset);
    }

    /// A gate that declares no floor admits every class, the weakest included.
    ///
    /// This is what "`Unset` migrates nobody" means in code: a caller that never states the
    /// field judges exactly as it did before the field existed.
    #[test]
    fn Test_An_Unset_Floor_Should_Admit_Every_Class()
    {
        for class in WEAKEST_FIRST
        {
            assert!(EvidenceFloor::Unset.Admits(class), "an unset floor admits {}", class.Label());
        }
    }

    /// The comparison is `EvidenceClass`'s own `Ord` and not a second ranking.
    ///
    /// Asserted as the whole cross-product rather than at one pair: a floor comparing with a
    /// table of its own would agree with the derived order at most pairs and disagree
    /// somewhere, and a single named pair could sit on the agreeing side of that.
    #[test]
    fn Test_The_Floor_Should_Follow_EvidenceClass_Own_Ordering()
    {
        for floor in WEAKEST_FIRST
        {
            for class in WEAKEST_FIRST
            {
                assert_eq!(
                    EvidenceFloor::AtLeast(floor).Admits(class),
                    class >= floor,
                    "a floor of {} judged {} against something other than EvidenceClass's own ordering",
                    floor.Label(),
                    class.Label()
                );
            }
        }
    }

    /// "Only mechanical findings may block" is a point on the same order, not a second axis.
    ///
    /// `OD-GATE-034` states it: a gate wanting that states `Approximate`, because
    /// `EvidenceClass::Is_Mechanical` is exactly "at or above `Approximate`". If the two ever
    /// part company, a repository that wrote the floor the record told it to write would be
    /// judging by something else.
    #[test]
    fn Test_A_Floor_At_Approximate_Should_Admit_Exactly_The_Mechanical_Classes()
    {
        let floor = EvidenceFloor::AtLeast(EvidenceClass::Approximate);

        for class in WEAKEST_FIRST
        {
            assert_eq!(floor.Admits(class), class.Is_Mechanical(), "{} is not judged the same way by both", class.Label());
        }
    }

    /// Only a stated floor has a class to cite.
    #[test]
    fn Test_Required_Should_Name_A_Class_Only_When_One_Is_Stated()
    {
        assert_eq!(EvidenceFloor::Unset.Required(), None);
        assert_eq!(EvidenceFloor::AtLeast(EvidenceClass::Derived).Required(), Some(EvidenceClass::Derived));
    }
}
