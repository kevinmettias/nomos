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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Assurance, FactVariant, IncrementalGranularity};

    fn Sample() -> Guarantee
    {
        return Guarantee::New(
            FactVariant::Syntactic,
            Assurance::Sound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );
    }

    #[test]
    fn Test_Of_Should_Be_Deterministic_For_The_Same_Guarantee()
    {
        assert_eq!(GuaranteeDigest::Of(&Sample()), GuaranteeDigest::Of(&Sample()));
    }

    #[test]
    fn Test_Digest_Should_Differ_For_A_Different_Guarantee()
    {
        let other = Guarantee::New(
            FactVariant::Approximate,
            Assurance::Unsound,
            Assurance::Unknown,
            IncrementalGranularity::File,
        );

        assert_ne!(GuaranteeDigest::Of(&Sample()).Digest(), GuaranteeDigest::Of(&other).Digest());
    }
}
