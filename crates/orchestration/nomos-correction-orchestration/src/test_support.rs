//! Fixtures shared by this crate's own unit tests -- the plans a scheduling case is made
//! of, and the one refusal the substrate produces that no real correction family can.

use nomos_contracts::{Assurance, FactVariant, Guarantee, IncrementalGranularity, ProviderId};
use nomos_corrections::{
    ChangeSet, CorrectionCandidate, CorrectionClass, CorrectionPlan, DerivedProvenance, Edit, ReadWriteResolution, ReadWriteSet,
    UnresolvedAccess, WavePartition,
};

/// The rewrite a fixture plan performs: which file, what it holds now, and what it should
/// hold afterwards. Named fields rather than three adjacent `&str` positions, which a call
/// site could transpose without the compiler objecting.
pub(crate) struct Rewrite<'text>
{
    pub path: &'text str,
    pub from: &'text str,
    pub to: &'text str,
}

/// A one-candidate plan that rewrites `path` from `before` to `after`.
pub(crate) fn Plan_Rewriting(path: &str, before: &str, after: &str) -> CorrectionPlan
{
    return Plan_Of(Candidate_Rewriting(&Rewrite {
        path,
        from: before,
        to: after,
    }));
}

/// A one-candidate plan that performs `rewrite` and declares it also reads `reads` at the
/// artifact tier without writing it.
///
/// The read-write conflict `COR-EXEC-003` is written for, and the only way two scheduled
/// plans can conflict without both writing one path. Neither shipped correction family
/// declares a read set, so a fixture is where this shape exists today.
pub(crate) fn Plan_Rewriting_While_Reading(rewrite: &Rewrite<'_>, reads: &str) -> CorrectionPlan
{
    let read = ReadWriteSet::Declared(ReadWriteResolution::Artifact, vec![reads.to_owned()]);

    return Plan_Of(Candidate_Rewriting(rewrite).With_Read(read));
}

fn Candidate_Rewriting(rewrite: &Rewrite<'_>) -> CorrectionCandidate
{
    let edit = Edit::New(rewrite.path, Some(rewrite.from.to_owned()), Some(rewrite.to.to_owned()));

    return CorrectionCandidate::New(format!("rewrite {}", rewrite.path), ChangeSet::Empty().With(edit), CorrectionClass::Mechanical, vec![]);
}

fn Plan_Of(candidate: CorrectionCandidate) -> CorrectionPlan
{
    return CorrectionPlan::New(vec![candidate]).expect("one candidate is a valid plan");
}

/// A plan whose candidate carries a derived read/write set its provider does not claim is
/// complete -- the one shape `nomos_corrections::WavePartition::Of` refuses to place.
///
/// Neither shipped correction family can build one: both declare their footprint at the
/// artifact tier from the change set itself, and a declared set is resolved by definition.
/// So the refusal path has to be reached with a plan built here, and reaching it is the
/// point -- a refusal nothing exercises is a refusal nobody knows still works.
pub(crate) fn Unresolved_Plan() -> CorrectionPlan
{
    let guarantee = Guarantee::New(FactVariant::SemanticallyResolved, Assurance::Sound, Assurance::Unknown, IncrementalGranularity::File);
    let provenance = DerivedProvenance::New(ProviderId::New("test"), guarantee, "low", "invalidated when the dependency graph changes");
    let set = ReadWriteSet::Derived(ReadWriteResolution::Dependency, vec!["nomos-a -> nomos-b".to_owned()], provenance);

    let edit = Edit::New("unresolved.rs", Some("before".to_owned()), Some("after".to_owned()));
    let candidate =
        CorrectionCandidate::New("unresolved", ChangeSet::Empty().With(edit), CorrectionClass::Mechanical, vec![]).With_Read_Write(set);

    return Plan_Of(candidate);
}

/// The refusal [`Unresolved_Plan`] earns from the substrate, unchanged.
pub(crate) fn Unresolved_Refusal() -> UnresolvedAccess
{
    return WavePartition::Of(&[Unresolved_Plan()]).expect_err("an incomplete derived set is unresolved");
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Plan_Rewriting_Should_Touch_Exactly_The_Given_Path()
    {
        let plan = Plan_Rewriting("a.rs", "old", "new");

        assert_eq!(plan.Candidates().len(), 1);
        let candidate = plan.Candidates().first().expect("the plan holds its one candidate");
        assert_eq!(candidate.Change().Touched(), std::collections::BTreeSet::from(["a.rs"]));
    }

    #[test]
    fn Test_Plan_Rewriting_While_Reading_Should_Declare_One_Artifact_Tier_Read()
    {
        let plan = Plan_Rewriting_While_Reading(&Rewrite { path: "a.rs", from: "old", to: "new" }, "b.rs");

        let candidate = plan.Candidates().first().expect("the plan holds its one candidate");
        assert_eq!(candidate.Reads(), [ReadWriteSet::Declared(ReadWriteResolution::Artifact, vec!["b.rs".to_owned()])]);
    }

    #[test]
    fn Test_Unresolved_Refusal_Should_Name_The_Tier_Its_Provider_Would_Not_Vouch_For()
    {
        let refusal = Unresolved_Refusal();

        assert_eq!(refusal.Resolution(), ReadWriteResolution::Dependency);
        assert_eq!(refusal.Completeness(), Assurance::Unknown);
    }
}
