//! What one plan declares it reads and writes, folded from every candidate's declared
//! sets into the two sets `COR-EXEC-003` compares.

mod unresolved_access;

pub use unresolved_access::UnresolvedAccess;

use crate::{CorrectionCandidate, CorrectionPlan, Overlap, OverlapClass, ReadWriteResolution, ReadWriteSet};
use std::collections::BTreeSet;

/// One declared subject: an entry at the tier it was declared at.
///
/// `a.rs` at [`ReadWriteResolution::Artifact`] and `a.rs` at
/// [`ReadWriteResolution::Symbol`] are two subjects, because nothing here resolves one
/// tier into another -- a compatibility judgment is computed from what plans declare and
/// never inferred past it. Keyed tier-first so that a set of subjects orders by tier and
/// then by spelling.
type Subject = (ReadWriteResolution, String);

/// The read set and write set one plan declares, at every tier its candidates declared
/// anything at.
///
/// `COR-EXEC-003`: "Independent corrections may execute in parallel waves only when their
/// read/write sets ... are compatible. Unknown independence shall not be treated as safe
/// parallelism." This is the resolved half of that precondition for one plan. Every
/// subject on a candidate's [`CorrectionCandidate::Read_Write`] is both read and written,
/// because [`crate::StagedPlan`] asserts a path's prior content before replacing it; every
/// subject on [`CorrectionCandidate::Reads`] is read only. So `reads` always contains
/// `writes`, and a subject in `reads` but not in `writes` is the one shape a read-only
/// claim adds.
///
/// Crate-private: a caller asks [`crate::Compatibility::Of`] or
/// [`crate::WavePartition::Of`] over plans, and this is what both compute from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlanAccess
{
    reads: BTreeSet<Subject>,
    writes: BTreeSet<Subject>,
}

impl PlanAccess
{
    /// Folds every set `plan`'s candidates declare. `position` is where `plan` sits in
    /// the caller's input, carried onto the refusal so a caller holding several plans can
    /// tell which one could not be resolved.
    ///
    /// # Errors
    ///
    /// Returns [`UnresolvedAccess`] for the first set, in candidate order, whose
    /// provenance does not claim completeness -- [`Assert_Resolved`] says why completeness
    /// and only completeness.
    pub(crate) fn Of(position: usize, plan: &CorrectionPlan) -> Result<Self, UnresolvedAccess>
    {
        let mut access = Self {
            reads: BTreeSet::new(),
            writes: BTreeSet::new(),
        };

        for candidate in plan.Candidates()
        {
            access.Absorb(position, candidate)?;
        }

        return Ok(access);
    }

    fn Absorb(&mut self, position: usize, candidate: &CorrectionCandidate) -> Result<(), UnresolvedAccess>
    {
        for set in candidate.Read_Write()
        {
            Assert_Resolved(position, candidate, set)?;
            self.writes.extend(Subjects_Of(set));
            self.reads.extend(Subjects_Of(set));
        }

        for set in candidate.Reads()
        {
            Assert_Resolved(position, candidate, set)?;
            self.reads.extend(Subjects_Of(set));
        }

        return Ok(());
    }

    /// Every subject `self` and `other` cannot both touch in one wave, each named once at
    /// its strongest class, ordered by tier and then by spelling.
    ///
    /// A subject both plans only read is not here: two readers disturb nothing. Since
    /// `reads` contains `writes`, a subject both write is also a subject each reads while
    /// the other writes, and it is reported once, as the write-write it is.
    pub(crate) fn Overlaps_With(&self, other: &Self) -> Vec<Overlap>
    {
        let crossing: BTreeSet<&Subject> = self.writes.intersection(&other.reads).chain(self.reads.intersection(&other.writes)).collect();

        return crossing.into_iter().map(|subject| return self.Overlap_On(other, subject)).collect();
    }

    pub(crate) fn Conflicts_With(&self, other: &Self) -> bool
    {
        return !self.Overlaps_With(other).is_empty();
    }

    fn Overlap_On(&self, other: &Self, subject: &Subject) -> Overlap
    {
        let (resolution, entry) = subject;
        let class = if self.writes.contains(subject) && other.writes.contains(subject)
        {
            OverlapClass::WriteWrite
        }
        else
        {
            OverlapClass::ReadWrite
        };

        return Overlap::New(*resolution, entry.clone(), class);
    }
}

fn Subjects_Of(set: &ReadWriteSet) -> impl Iterator<Item = Subject> + '_
{
    return set.Entries().iter().map(move |entry| return (set.Resolution(), entry.clone()));
}

/// A declared set is resolved by definition: its author asserted it. A derived set is
/// resolved only when its provider claims completeness, because an incomplete set can be
/// missing the one subject that would have made two plans conflict, and a `Compatible`
/// verdict built on it is exactly the "unknown independence" `COR-EXEC-003` forbids
/// treating as safe.
///
/// Soundness is deliberately not required. An unsound set can only name subjects the plan
/// does not really touch, which over-reports conflicts and serializes more than necessary
/// -- the safe direction. [`nomos_contracts::Assurance::Satisfies_Requirement`] is the one
/// place this workspace says `Unknown` is not a pass, and this reuses it rather than
/// deciding again.
fn Assert_Resolved(position: usize, candidate: &CorrectionCandidate, set: &ReadWriteSet) -> Result<(), UnresolvedAccess>
{
    let Some(provenance) = set.Derived_Provenance()
    else
    {
        return Ok(());
    };

    let completeness = provenance.Guarantee().completeness;
    if completeness.Satisfies_Requirement()
    {
        return Ok(());
    }

    return Err(UnresolvedAccess::New(position, candidate.Id(), set.Resolution(), completeness));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::{Candidate_Writing, Derived_Set_With_Completeness, Plan_Of, Plan_Writing, Reading};
    use nomos_contracts::Assurance;

    const FIRST_PLAN: usize = 0;
    const THIRD_PLAN: usize = 2;

    #[test]
    fn Test_Of_Should_Enter_Every_Read_Write_Subject_As_Both_Read_And_Written()
    {
        let plan = Plan_Writing(&["a.rs", "b.rs"]);

        let access = PlanAccess::Of(FIRST_PLAN, &plan).expect("a declared set is resolved");

        let expected: BTreeSet<Subject> =
            BTreeSet::from([(ReadWriteResolution::Artifact, "a.rs".to_owned()), (ReadWriteResolution::Artifact, "b.rs".to_owned())]);
        assert_eq!(access.writes, expected);
        assert_eq!(access.reads, expected);
    }

    #[test]
    fn Test_Of_Should_Enter_A_Declared_Read_As_Read_Only()
    {
        let plan = Plan_Of(Reading(Candidate_Writing(&["b.rs"]), "a.rs"));

        let access = PlanAccess::Of(FIRST_PLAN, &plan).expect("a declared set is resolved");

        assert!(access.reads.contains(&(ReadWriteResolution::Artifact, "a.rs".to_owned())));
        assert!(!access.writes.contains(&(ReadWriteResolution::Artifact, "a.rs".to_owned())));
    }

    #[test]
    fn Test_Of_Should_Refuse_A_Derived_Set_Whose_Completeness_Is_Unknown()
    {
        let candidate = Candidate_Writing(&["a.rs"]).With_Read_Write(Derived_Set_With_Completeness(Assurance::Unknown));
        let plan = Plan_Of(candidate.clone());

        let refusal = PlanAccess::Of(THIRD_PLAN, &plan).expect_err("an incomplete derived set is unresolved");

        assert_eq!(refusal.Plan(), THIRD_PLAN);
        assert_eq!(refusal.Candidate(), candidate.Id());
        assert_eq!(refusal.Resolution(), ReadWriteResolution::Dependency);
        assert_eq!(refusal.Completeness(), Assurance::Unknown);
    }

    #[test]
    fn Test_Of_Should_Refuse_A_Derived_Read_Whose_Completeness_Is_Unsound()
    {
        let plan = Plan_Of(Candidate_Writing(&["a.rs"]).With_Read(Derived_Set_With_Completeness(Assurance::Unsound)));

        let refusal = PlanAccess::Of(FIRST_PLAN, &plan).expect_err("an incomplete derived set is unresolved");

        assert_eq!(refusal.Completeness(), Assurance::Unsound);
    }

    #[test]
    fn Test_Of_Should_Accept_A_Derived_Set_Whose_Completeness_Is_Sound()
    {
        let plan = Plan_Of(Candidate_Writing(&["a.rs"]).With_Read_Write(Derived_Set_With_Completeness(Assurance::Sound)));

        let access = PlanAccess::Of(FIRST_PLAN, &plan).expect("a complete derived set is resolved");

        assert!(access.writes.contains(&(ReadWriteResolution::Dependency, "nomos-corrections -> nomos-workspace".to_owned())));
    }

    #[test]
    fn Test_Overlaps_With_Should_Report_Nothing_When_Both_Only_Read_One_Subject()
    {
        let first = PlanAccess::Of(FIRST_PLAN, &Plan_Of(Reading(Candidate_Writing(&["b.rs"]), "shared.rs"))).expect("resolved");
        let second = PlanAccess::Of(FIRST_PLAN, &Plan_Of(Reading(Candidate_Writing(&["c.rs"]), "shared.rs"))).expect("resolved");

        assert!(first.Overlaps_With(&second).is_empty());
        assert!(!first.Conflicts_With(&second));
    }

    #[test]
    fn Test_Overlaps_With_Should_Order_By_Tier_Then_Spelling()
    {
        let symbol = ReadWriteSet::Declared(ReadWriteResolution::Symbol, vec!["a::item".to_owned()]);
        let first = PlanAccess::Of(FIRST_PLAN, &Plan_Of(Candidate_Writing(&["z.rs", "b.rs"]).With_Read_Write(symbol.clone()))).expect("resolved");
        let second = PlanAccess::Of(FIRST_PLAN, &Plan_Of(Candidate_Writing(&["b.rs", "z.rs"]).With_Read_Write(symbol))).expect("resolved");

        let subjects: Vec<Subject> =
            first.Overlaps_With(&second).iter().map(|overlap| return (overlap.Resolution(), overlap.Subject().to_owned())).collect();

        assert_eq!(
            subjects,
            [
                (ReadWriteResolution::Artifact, "b.rs".to_owned()),
                (ReadWriteResolution::Artifact, "z.rs".to_owned()),
                (ReadWriteResolution::Symbol, "a::item".to_owned()),
            ]
        );
    }
}
