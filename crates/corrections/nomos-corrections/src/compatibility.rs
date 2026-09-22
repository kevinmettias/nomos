//! Whether two plans may share a wave, decided from what they declare and nothing else.

mod overlap;
mod overlap_class;

pub use overlap::Overlap;
pub use overlap_class::OverlapClass;

use crate::plan_access::PlanAccess;
use crate::{CorrectionPlan, UnresolvedAccess};

/// Where `first` and `second` sit in [`Compatibility::Of`]'s two-plan input -- what an
/// [`UnresolvedAccess`] from that call reports as its plan.
const FIRST: usize = 0;
const SECOND: usize = 1;

/// Whether two plans may be staged in one wave.
///
/// `COR-EXEC-002` names textual conflict as the first edge of the correction execution
/// graph, and `COR-EXEC-003` makes compatible read/write sets the precondition for a
/// parallel wave; this is that edge for one pair, computed only from the sets the plans'
/// candidates declare (`COR-EXEC-001`). Nothing here opens a file or diffs content: two
/// plans whose declared subjects are disjoint are `Compatible` even if their edits would
/// in fact collide, and a caller that knows more says so through
/// [`crate::CorrectionCandidate::With_Read_Write`] or
/// [`crate::CorrectionCandidate::With_Read`] rather than having it inferred here.
///
/// `COR-003`'s "compatible set" is the same question at a different grain:
/// [`CorrectionPlan::New`] refuses two *candidates* touching one path inside a plan, and
/// this judges two *plans* against each other -- what a plan already guarantees about
/// itself, asked across plans. It does not replace that refusal, because a plan is what
/// stages atomically and a wave is not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Compatibility
{
    /// No subject one plan writes is a subject the other reads or writes.
    Compatible,
    /// At least one subject is, and these are all of them, each at its strongest class.
    Conflicting
    {
        overlaps: Vec<Overlap>,
    },
}

impl Compatibility
{
    /// Judges `first` against `second`. The answer is symmetric in what it reports; only
    /// the positions an [`UnresolvedAccess`] carries depend on which was named first.
    ///
    /// # Errors
    ///
    /// Returns [`UnresolvedAccess`] when either plan's read or write set is unresolved --
    /// `first` before `second`, reported at position `0` or `1` respectively -- rather
    /// than judging an unknown footprint compatible with anything (`COR-EXEC-003`).
    pub fn Of(first: &CorrectionPlan, second: &CorrectionPlan) -> Result<Self, UnresolvedAccess>
    {
        let first_access = PlanAccess::Of(FIRST, first)?;
        let second_access = PlanAccess::Of(SECOND, second)?;

        let overlaps = first_access.Overlaps_With(&second_access);
        if overlaps.is_empty()
        {
            return Ok(Self::Compatible);
        }

        return Ok(Self::Conflicting { overlaps });
    }

    #[must_use]
    pub const fn Is_Compatible(&self) -> bool
    {
        return matches!(self, Self::Compatible);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::{Candidate_Writing, Derived_Set_With_Completeness, Plan_Of, Plan_Writing, Reading};
    use crate::{ReadWriteResolution, ReadWriteSet};
    use nomos_contracts::Assurance;

    fn Overlaps_Of(judgment: Compatibility) -> Vec<Overlap>
    {
        return match judgment
        {
            Compatibility::Compatible => panic!("expected a conflict"),
            Compatibility::Conflicting { overlaps } => overlaps,
        };
    }

    #[test]
    fn Test_Of_Should_Answer_Compatible_When_No_Subject_Is_Shared()
    {
        let judgment = Compatibility::Of(&Plan_Writing(&["a.rs"]), &Plan_Writing(&["b.rs"])).expect("declared sets are resolved");

        assert_eq!(judgment, Compatibility::Compatible);
        assert!(judgment.Is_Compatible());
    }

    #[test]
    fn Test_Of_Should_Class_A_Path_Both_Plans_Write_As_Write_Write()
    {
        let judgment = Compatibility::Of(&Plan_Writing(&["a.rs"]), &Plan_Writing(&["a.rs", "b.rs"])).expect("declared sets are resolved");

        assert!(!judgment.Is_Compatible());
        assert_eq!(Overlaps_Of(judgment), [Overlap::New(ReadWriteResolution::Artifact, "a.rs", OverlapClass::WriteWrite)]);
    }

    #[test]
    fn Test_Of_Should_Class_A_Read_Of_What_The_Other_Writes_As_Read_Write()
    {
        let reading = Plan_Of(Reading(Candidate_Writing(&["b.rs"]), "a.rs"));

        let judgment = Compatibility::Of(&reading, &Plan_Writing(&["a.rs"])).expect("declared sets are resolved");

        assert_eq!(Overlaps_Of(judgment), [Overlap::New(ReadWriteResolution::Artifact, "a.rs", OverlapClass::ReadWrite)]);
    }

    #[test]
    fn Test_Of_Should_Class_A_Write_Of_What_The_Other_Reads_As_Read_Write()
    {
        let reading = Plan_Of(Reading(Candidate_Writing(&["b.rs"]), "a.rs"));

        let judgment = Compatibility::Of(&Plan_Writing(&["a.rs"]), &reading).expect("declared sets are resolved");

        assert_eq!(Overlaps_Of(judgment), [Overlap::New(ReadWriteResolution::Artifact, "a.rs", OverlapClass::ReadWrite)]);
    }

    #[test]
    fn Test_Of_Should_Not_Conflict_Two_Plans_That_Only_Read_One_Subject()
    {
        let first = Plan_Of(Reading(Candidate_Writing(&["b.rs"]), "shared.rs"));
        let second = Plan_Of(Reading(Candidate_Writing(&["c.rs"]), "shared.rs"));

        let judgment = Compatibility::Of(&first, &second).expect("declared sets are resolved");

        assert_eq!(judgment, Compatibility::Compatible);
    }

    /// A subject both plans write, which one of them also declares it reads, is one
    /// overlap at the stronger class -- never a write-write and a read-write for one
    /// spelling.
    #[test]
    fn Test_Of_Should_Name_A_Subject_Once_At_Its_Strongest_Class()
    {
        let writing_and_reading = Plan_Of(Reading(Candidate_Writing(&["a.rs"]), "a.rs"));

        let judgment = Compatibility::Of(&writing_and_reading, &Plan_Writing(&["a.rs"])).expect("declared sets are resolved");

        assert_eq!(Overlaps_Of(judgment), [Overlap::New(ReadWriteResolution::Artifact, "a.rs", OverlapClass::WriteWrite)]);
    }

    /// One spelling at two tiers is two subjects: nothing here resolves a symbol into the
    /// artifact that holds it, and pretending to would be inferring past what was declared.
    #[test]
    fn Test_Of_Should_Keep_The_Same_Spelling_At_Two_Tiers_Apart()
    {
        let symbol_tier = ReadWriteSet::Declared(ReadWriteResolution::Symbol, vec!["a.rs".to_owned()]);
        let at_symbol = Plan_Of(Candidate_Writing(&["b.rs"]).With_Read_Write(symbol_tier));

        let judgment = Compatibility::Of(&at_symbol, &Plan_Writing(&["a.rs"])).expect("declared sets are resolved");

        assert_eq!(judgment, Compatibility::Compatible);
    }

    #[test]
    fn Test_Of_Should_List_Every_Overlap_In_Tier_Then_Spelling_Order()
    {
        let first = Plan_Writing(&["z.rs", "a.rs"]);
        let second = Plan_Writing(&["a.rs", "z.rs"]);

        let overlaps = Overlaps_Of(Compatibility::Of(&first, &second).expect("declared sets are resolved"));

        assert_eq!(
            overlaps,
            [
                Overlap::New(ReadWriteResolution::Artifact, "a.rs", OverlapClass::WriteWrite),
                Overlap::New(ReadWriteResolution::Artifact, "z.rs", OverlapClass::WriteWrite),
            ]
        );
    }

    #[test]
    fn Test_Of_Should_Refuse_When_The_Second_Plans_Set_Is_Unresolved()
    {
        let unresolved = Plan_Of(Candidate_Writing(&["b.rs"]).With_Read_Write(Derived_Set_With_Completeness(Assurance::Unknown)));

        let refusal = Compatibility::Of(&Plan_Writing(&["a.rs"]), &unresolved).expect_err("an incomplete derived set is unresolved");

        assert_eq!(refusal.Plan(), SECOND);
        assert_eq!(refusal.Resolution(), ReadWriteResolution::Dependency);
    }

    #[test]
    fn Test_Of_Should_Refuse_When_The_First_Plans_Set_Is_Unresolved()
    {
        let unresolved = Plan_Of(Candidate_Writing(&["b.rs"]).With_Read(Derived_Set_With_Completeness(Assurance::Unknown)));

        let refusal = Compatibility::Of(&unresolved, &Plan_Writing(&["a.rs"])).expect_err("an incomplete derived set is unresolved");

        assert_eq!(refusal.Plan(), FIRST);
    }

    #[test]
    fn Test_Of_Should_Judge_A_Complete_Derived_Set_Like_A_Declared_One()
    {
        let first = Plan_Of(Candidate_Writing(&["a.rs"]).With_Read_Write(Derived_Set_With_Completeness(Assurance::Sound)));
        let second = Plan_Of(Candidate_Writing(&["b.rs"]).With_Read_Write(Derived_Set_With_Completeness(Assurance::Sound)));

        let overlaps = Overlaps_Of(Compatibility::Of(&first, &second).expect("a complete derived set is resolved"));

        assert_eq!(overlaps, [Overlap::New(ReadWriteResolution::Dependency, "nomos-corrections -> nomos-workspace", OverlapClass::WriteWrite)]);
    }

    #[test]
    fn Test_Is_Compatible_Should_Be_False_For_A_Conflict()
    {
        let conflicting = Compatibility::Conflicting {
            overlaps: vec![Overlap::New(ReadWriteResolution::Artifact, "a.rs", OverlapClass::WriteWrite)],
        };

        assert!(!conflicting.Is_Compatible());
    }
}
