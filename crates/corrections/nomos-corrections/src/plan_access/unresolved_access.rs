//! The refusal a plan earns when a set it declares cannot be trusted to be complete.

use crate::{CorrectionId, ReadWriteResolution};
use nomos_contracts::Assurance;

/// A plan whose read or write set is unresolved: one of its candidates carries a derived
/// [`crate::ReadWriteSet`] whose provider does not claim completeness, so nothing can say
/// what the plan does *not* touch, and so nothing can say it is independent of anything.
///
/// `COR-EXEC-003`: "Unknown independence shall not be treated as safe parallelism."
/// [`crate::Compatibility::Of`] and [`crate::WavePartition::Of`] return this rather than
/// judge the plan compatible with anything -- a typed refusal a caller can act on
/// (serialize, broaden validation, or require review: `COR-EXEC-005`'s own three
/// alternatives to optimistic staging), never a guess and never a panic.
///
/// `plan` is where the plan sat in the caller's input, because a [`crate::CorrectionPlan`]
/// has no identity of its own; `candidate` is which of its candidates carried the set, and
/// `resolution` and `completeness` are what that set claimed about itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnresolvedAccess
{
    plan: usize,
    candidate: CorrectionId,
    resolution: ReadWriteResolution,
    completeness: Assurance,
}

impl UnresolvedAccess
{
    #[must_use]
    pub(crate) const fn New(plan: usize, candidate: CorrectionId, resolution: ReadWriteResolution, completeness: Assurance) -> Self
    {
        return Self {
            plan,
            candidate,
            resolution,
            completeness,
        };
    }

    /// The position, in the input the refusing call was given, of the plan that could not
    /// be resolved.
    #[must_use]
    pub const fn Plan(&self) -> usize
    {
        return self.plan;
    }

    #[must_use]
    pub const fn Candidate(&self) -> CorrectionId
    {
        return self.candidate;
    }

    /// The tier of the derived set that could not be trusted.
    #[must_use]
    pub const fn Resolution(&self) -> ReadWriteResolution
    {
        return self.resolution;
    }

    /// What the set's provider claimed about completeness -- never
    /// [`Assurance::Sound`], or this refusal would not exist.
    #[must_use]
    pub const fn Completeness(&self) -> Assurance
    {
        return self.completeness;
    }
}

impl core::fmt::Display for UnresolvedAccess
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(
            formatter,
            "plan {} cannot be placed: candidate {} carries a derived {}-tier read/write set whose \
             completeness is {}, so what the plan does not touch is unknown and its independence \
             from every other plan is unknown with it",
            self.plan, self.candidate, self.resolution, self.completeness
        );
    }
}

impl std::error::Error for UnresolvedAccess
{}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::Digest128;

    const ARBITRARY_DIGEST_BYTE: u8 = 0x61;
    const SECOND_PLAN: usize = 1;

    fn Refusal() -> UnresolvedAccess
    {
        let candidate = CorrectionId::From_Digest(Digest128::From_Bytes([ARBITRARY_DIGEST_BYTE; Digest128::BYTE_LENGTH]));

        return UnresolvedAccess::New(SECOND_PLAN, candidate, ReadWriteResolution::Dependency, Assurance::Unknown);
    }

    #[test]
    fn Test_New_Should_Carry_Every_Field_It_Was_Given()
    {
        let refusal = Refusal();

        assert_eq!(refusal.Plan(), SECOND_PLAN);
        assert_eq!(refusal.Candidate(), Refusal().Candidate());
        assert_eq!(refusal.Resolution(), ReadWriteResolution::Dependency);
        assert_eq!(refusal.Completeness(), Assurance::Unknown);
    }

    #[test]
    fn Test_Plan_Should_Report_The_Input_Position()
    {
        assert_eq!(Refusal().Plan(), SECOND_PLAN);
    }

    #[test]
    fn Test_Candidate_Should_Report_Which_Candidate_Carried_The_Set()
    {
        let expected = CorrectionId::From_Digest(Digest128::From_Bytes([ARBITRARY_DIGEST_BYTE; Digest128::BYTE_LENGTH]));

        assert_eq!(Refusal().Candidate(), expected);
    }

    #[test]
    fn Test_Resolution_Should_Report_The_Sets_Tier()
    {
        assert_eq!(Refusal().Resolution(), ReadWriteResolution::Dependency);
    }

    #[test]
    fn Test_Completeness_Should_Report_What_The_Provider_Claimed()
    {
        assert_eq!(Refusal().Completeness(), Assurance::Unknown);
    }

    #[test]
    fn Test_Display_Should_Name_The_Plan_The_Tier_And_The_Claim()
    {
        let rendered = Refusal().to_string();

        assert!(rendered.contains("plan 1 "), "{rendered}");
        assert!(rendered.contains("dependency-tier"), "{rendered}");
        assert!(rendered.contains("completeness is Unknown"), "{rendered}");
    }
}
