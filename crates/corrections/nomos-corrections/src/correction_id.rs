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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The one byte each of these cases fills a digest with. Two cases need two digests
    /// that differ, so they use two different bytes.
    const CARRIED_DIGEST_BYTE: u8 = 7;
    const FIRST_DIGEST_BYTE: u8 = 1;
    const SECOND_DIGEST_BYTE: u8 = 2;

    #[test]
    fn Test_From_Digest_Should_Construct_An_Id_Carrying_That_Digest()
    {
        let digest = Digest128::From_Bytes([CARRIED_DIGEST_BYTE; Digest128::BYTE_LENGTH]);
        let id = CorrectionId::From_Digest(digest);

        assert_eq!(id.Digest(), digest);
    }

    #[test]
    fn Test_Digest_Should_Return_What_The_Id_Was_Constructed_From()
    {
        let first = Digest128::From_Bytes([FIRST_DIGEST_BYTE; Digest128::BYTE_LENGTH]);
        let second = Digest128::From_Bytes([SECOND_DIGEST_BYTE; Digest128::BYTE_LENGTH]);

        assert_ne!(CorrectionId::From_Digest(first).Digest(), CorrectionId::From_Digest(second).Digest());
    }
}
