//! The digest of the guarantee a fact was produced at.

use nomos_contracts::Guarantee;
use nomos_contracts::Digest128;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GuaranteeDigest(Digest128);

impl GuaranteeDigest
{
    #[must_use]
    pub fn Of(guarantee: &Guarantee) -> Self
    {
        use nomos_model::Digest_Of_Parts;

        return Self(Digest_Of_Parts(&[
            &[guarantee.variant as u8],
            &[guarantee.soundness as u8],
            &[guarantee.completeness as u8],
            &[guarantee.incremental as u8],
        ]));
    }

    #[must_use]
    pub const fn Digest(self) -> Digest128
    {
        return self.0;
    }
}

impl core::fmt::Display for GuaranteeDigest
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return self.0.fmt(formatter);
    }
}
