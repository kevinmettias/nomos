//! One proposed correction: what it is for, and what it would change.

use nomos_contracts::Digest128;

mod correction_candidate;

pub use correction_candidate::CorrectionCandidate;

/// Identity of a [`CorrectionCandidate`], derived from what it says and what it would do.
///
/// Content-derived rather than authored or counted, so that proposing the same correction
/// twice — from two rules, or from the same rule over two runs — is one candidate rather
/// than two competing for the same path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CorrectionId(Digest128);

impl CorrectionId
{
    #[must_use]
    pub const fn From_Digest(digest: Digest128) -> Self
    {
        return Self(digest);
    }

    #[must_use]
    pub const fn Digest(&self) -> Digest128
    {
        return self.0;
    }
}

impl core::fmt::Display for CorrectionId
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return self.0.fmt(formatter);
    }
}
