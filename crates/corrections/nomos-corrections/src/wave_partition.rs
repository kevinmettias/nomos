//! An ordered grouping of plans such that no wave holds a conflicting pair.

use crate::plan_access::PlanAccess;
use crate::{CorrectionPlan, UnresolvedAccess};

/// Where a plan that conflicts with no predecessor goes.
const FIRST_WAVE: usize = 0;

/// `COR-EXEC-003`'s parallel waves over a slice of plans: each wave lists the input
/// positions of the plans it holds, ascending; no two plans in one wave conflict; and
/// every conflicting pair sits in two waves with the earlier input plan in the earlier
/// wave.
///
/// The partition is a pure function of the input order. Each plan, taken in input order,
/// is placed in the wave after the last wave holding a predecessor it conflicts with, or
/// in the first wave when it conflicts with no predecessor. So the same slice always
/// yields the same waves, and reordering the slice is the only way to change them. This is
/// not a search for the fewest waves: minimising wave count is a scheduling objective the
/// corpus assigns to a planner with more inputs than declared sets (`COR-EXEC-002`'s eight
/// edge kinds, `COR-EXEC-006`'s rollback boundaries), and choosing an objective here would
/// decide it in the wrong place.
///
/// Positions rather than plans, because `COR-EXEC-008`'s execution record preserves
/// "parallel-wave membership" beside the planned graph and the actual order, and a
/// position is what ties a wave back to the plan the caller handed in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WavePartition
{
    waves: Vec<Vec<usize>>,
}

impl WavePartition
{
    /// Partitions `plans` by their declared read and write sets alone.
    ///
    /// # Errors
    ///
    /// Returns [`UnresolvedAccess`] for the first plan, in input order, whose read or
    /// write set is unresolved, and partitions nothing: `COR-EXEC-003` says unknown
    /// independence is not safe parallelism, and waves built around a hole would be
    /// exactly that.
    pub fn Of(plans: &[CorrectionPlan]) -> Result<Self, UnresolvedAccess>
    {
        let accesses = Resolved(plans)?;
        let mut waves: Vec<Vec<usize>> = Vec::new();

        for (position, access) in accesses.iter().enumerate()
        {
            let wave = Wave_After_Last_Conflict(&waves, &accesses, access);
            Place(&mut waves, wave, position);
        }

        return Ok(Self { waves });
    }

    /// Every wave in order, each listing the input positions of its plans, ascending.
    #[must_use]
    pub fn Waves(&self) -> &[Vec<usize>]
    {
        return &self.waves;
    }

    /// Which wave the plan at input `position` landed in, or `None` for a position the
    /// input did not have.
    #[must_use]
    pub fn Wave_Of(&self, position: usize) -> Option<usize>
    {
        return self.waves.iter().position(|wave| return wave.contains(&position));
    }
}

fn Resolved(plans: &[CorrectionPlan]) -> Result<Vec<PlanAccess>, UnresolvedAccess>
{
    return plans.iter().enumerate().map(|(position, plan)| return PlanAccess::Of(position, plan)).collect();
}

/// The wave `access` must go in: the one after the last wave holding a plan it conflicts
/// with, or the first when none does. The last, not the first, because a plan conflicting
/// with two predecessors in two different waves must come after both.
fn Wave_After_Last_Conflict(waves: &[Vec<usize>], accesses: &[PlanAccess], access: &PlanAccess) -> usize
{
    let last = waves.iter().rposition(|wave| return wave.iter().any(|member| return Conflicts(accesses, *member, access)));

    return match last
    {
        None => FIRST_WAVE,
        Some(last) => last.saturating_add(1),
    };
}

fn Conflicts(accesses: &[PlanAccess], member: usize, access: &PlanAccess) -> bool
{
    return accesses.get(member).is_some_and(|other| return other.Conflicts_With(access));
}

/// `wave` is either an existing wave or the one just past the end -- it is the wave after
/// one that exists, or the first -- so a wave that does not exist yet is always the next
/// one to push.
fn Place(waves: &mut Vec<Vec<usize>>, wave: usize, position: usize)
{
    match waves.get_mut(wave)
    {
        Some(members) => members.push(position),
        None => waves.push(vec![position]),
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::{Candidate_Writing, Derived_Set_With_Completeness, Plan_Of, Plan_Writing, Reading};
    use crate::Compatibility;
    use nomos_contracts::Assurance;

    const FIRST: usize = 0;
    const SECOND: usize = 1;
    const THIRD: usize = 2;
    const FOURTH: usize = 3;

    /// Every pair the partition put in one wave is compatible, and every conflicting pair
    /// has the earlier input plan in the strictly earlier wave -- the two properties the
    /// type claims, checked over every pair rather than the ones a case happened to name.
    fn Assert_Sound(partition: &WavePartition, plans: &[CorrectionPlan])
    {
        for (earlier, earlier_plan) in plans.iter().enumerate()
        {
            for (later, later_plan) in plans.iter().enumerate().skip(earlier.saturating_add(1))
            {
                let earlier_wave = partition.Wave_Of(earlier).expect("every input plan is placed");
                let later_wave = partition.Wave_Of(later).expect("every input plan is placed");
                let compatible = Compatibility::Of(earlier_plan, later_plan).expect("resolved").Is_Compatible();

                assert!(compatible || earlier_wave < later_wave, "plans {earlier} and {later} conflict but sit in waves {earlier_wave} and {later_wave}");
            }
        }
    }

    /// The case the item names: three plans, exactly one conflicting pair, and the two
    /// compatible ones share the first wave while the conflicting successor waits.
    #[test]
    fn Test_Of_Should_Put_Three_Plans_With_One_Conflict_Into_Two_Waves()
    {
        let plans = [Plan_Writing(&["a.rs"]), Plan_Writing(&["b.rs"]), Plan_Writing(&["a.rs"])];

        let partition = WavePartition::Of(&plans).expect("declared sets are resolved");

        assert_eq!(partition.Waves(), [vec![FIRST, SECOND], vec![THIRD]]);
        Assert_Sound(&partition, &plans);
    }

    #[test]
    fn Test_Of_Should_Put_Pairwise_Compatible_Plans_In_One_Wave()
    {
        let plans = [Plan_Writing(&["a.rs"]), Plan_Writing(&["b.rs"]), Plan_Writing(&["c.rs"])];

        let partition = WavePartition::Of(&plans).expect("declared sets are resolved");

        assert_eq!(partition.Waves(), [vec![FIRST, SECOND, THIRD]]);
    }

    #[test]
    fn Test_Of_Should_Give_A_Chain_Of_Conflicts_One_Wave_Each()
    {
        let plans = [Plan_Writing(&["a.rs"]), Plan_Writing(&["a.rs", "b.rs"]), Plan_Writing(&["b.rs"])];

        let partition = WavePartition::Of(&plans).expect("declared sets are resolved");

        assert_eq!(partition.Waves(), [vec![FIRST], vec![SECOND], vec![THIRD]]);
        Assert_Sound(&partition, &plans);
    }

    /// A plan conflicting with predecessors in two different waves goes after the later
    /// one. Placing it after the first conflicting predecessor instead would seat it
    /// beside the second, in a wave with a plan it conflicts with.
    #[test]
    fn Test_Of_Should_Place_A_Plan_After_Its_Latest_Conflicting_Predecessor()
    {
        let plans = [
            Plan_Writing(&["a.rs"]),
            Plan_Writing(&["a.rs"]),
            Plan_Of(Reading(Candidate_Writing(&["b.rs"]), "a.rs")),
        ];

        let partition = WavePartition::Of(&plans).expect("declared sets are resolved");

        assert_eq!(partition.Waves(), [vec![FIRST], vec![SECOND], vec![THIRD]]);
        Assert_Sound(&partition, &plans);
    }

    #[test]
    fn Test_Of_Should_Separate_A_Read_Write_Conflict_Too()
    {
        let plans = [Plan_Writing(&["a.rs"]), Plan_Of(Reading(Candidate_Writing(&["b.rs"]), "a.rs"))];

        let partition = WavePartition::Of(&plans).expect("declared sets are resolved");

        assert_eq!(partition.Waves(), [vec![FIRST], vec![SECOND]]);
    }

    #[test]
    fn Test_Of_Should_Let_A_Later_Plan_Rejoin_An_Earlier_Wave_When_It_Conflicts_With_Nothing()
    {
        let plans = [Plan_Writing(&["a.rs"]), Plan_Writing(&["a.rs"]), Plan_Writing(&["c.rs"]), Plan_Writing(&["a.rs", "c.rs"])];

        let partition = WavePartition::Of(&plans).expect("declared sets are resolved");

        assert_eq!(partition.Waves(), [vec![FIRST, THIRD], vec![SECOND], vec![FOURTH]]);
        Assert_Sound(&partition, &plans);
    }

    #[test]
    fn Test_Of_Should_Be_Deterministic_For_The_Same_Input_Order()
    {
        let plans = [Plan_Writing(&["a.rs"]), Plan_Writing(&["b.rs"]), Plan_Writing(&["a.rs"]), Plan_Writing(&["b.rs", "c.rs"])];

        let first_run = WavePartition::Of(&plans).expect("declared sets are resolved");
        let second_run = WavePartition::Of(&plans).expect("declared sets are resolved");

        assert_eq!(first_run, second_run);
    }

    /// The input order is the only tie-break. The plan writing both `a.rs` and `c.rs`
    /// waits in the second wave when it is handed in last, and takes the first wave when
    /// it is handed in first -- the earlier input plan of a conflicting pair is always the
    /// one in the earlier wave, whichever plan that happens to be.
    #[test]
    fn Test_Of_Should_Follow_The_Input_Order_When_It_Is_Reversed()
    {
        let forward = [Plan_Writing(&["a.rs"]), Plan_Writing(&["b.rs"]), Plan_Writing(&["a.rs", "c.rs"])];
        let reversed = [Plan_Writing(&["a.rs", "c.rs"]), Plan_Writing(&["b.rs"]), Plan_Writing(&["a.rs"])];

        let forward_partition = WavePartition::Of(&forward).expect("declared sets are resolved");
        let reversed_partition = WavePartition::Of(&reversed).expect("declared sets are resolved");

        assert_eq!(forward_partition.Wave_Of(THIRD), Some(SECOND), "handed in last, the two-path plan waits");
        assert_eq!(reversed_partition.Wave_Of(FIRST), Some(FIRST_WAVE), "handed in first, the two-path plan goes first");
        assert_eq!(reversed_partition.Wave_Of(THIRD), Some(SECOND), "and the plan it conflicts with now waits instead");
        Assert_Sound(&reversed_partition, &reversed);
    }

    #[test]
    fn Test_Of_Should_Refuse_When_Any_Plans_Set_Is_Unresolved_And_Name_Its_Position()
    {
        let unresolved = Plan_Of(Candidate_Writing(&["c.rs"]).With_Read_Write(Derived_Set_With_Completeness(Assurance::Unknown)));
        let plans = [Plan_Writing(&["a.rs"]), Plan_Writing(&["b.rs"]), unresolved];

        let refusal = WavePartition::Of(&plans).expect_err("an incomplete derived set is unresolved");

        assert_eq!(refusal.Plan(), THIRD);
        assert_eq!(refusal.Completeness(), Assurance::Unknown);
    }

    #[test]
    fn Test_Of_Should_Partition_No_Plans_Into_No_Waves()
    {
        let partition = WavePartition::Of(&[]).expect("nothing is trivially resolved");

        assert!(partition.Waves().is_empty());
    }

    #[test]
    fn Test_Wave_Of_Should_Report_None_For_A_Position_The_Input_Did_Not_Have()
    {
        let partition = WavePartition::Of(&[Plan_Writing(&["a.rs"])]).expect("declared sets are resolved");

        assert_eq!(partition.Wave_Of(SECOND), None);
    }
}
