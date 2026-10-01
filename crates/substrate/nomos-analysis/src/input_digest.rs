use nomos_contracts::Digest128;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InputDigest(Digest128);

impl InputDigest
{
    #[must_use]
    pub fn Of(parts: &[&[u8]]) -> Self
    {
        use nomos_model::Digest_Of_Parts;

        return Self(Digest_Of_Parts(parts));
    }

    #[must_use]
    pub const fn From_Digest(digest: Digest128) -> Self
    {
        return Self(digest);
    }

    #[must_use]
    pub const fn Digest(self) -> Digest128
    {
        return self.0;
    }
}

impl core::fmt::Display for InputDigest
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

    /// An arbitrary byte fill that is neither extreme. The third seed in `Sample_Seeds`, `0`,
    /// is the identity and is left bare there.
    const ARBITRARY_SEED_BYTE: u8 = 7;

    /// The top of the `u8` range, so the round trip is shown for a byte whose bits are all
    /// set as well as for one that is not.
    const MAX_SEED_BYTE: u8 = 255;

    #[test]
    fn Test_Of_Should_Be_Deterministic_For_The_Same_Parts()
    {
        assert_eq!(InputDigest::Of(&[b"a", b"b"]), InputDigest::Of(&[b"a", b"b"]));
        assert_ne!(InputDigest::Of(&[b"a"]), InputDigest::Of(&[b"b"]));
    }

    #[test]
    fn Test_From_Digest_Should_Wrap_The_Given_Value_Unchanged()
    {
        for seed in Sample_Seeds()
        {
            let digest = Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);

            assert_eq!(InputDigest::From_Digest(digest).Digest(), digest);
        }
    }

    #[test]
    fn Test_Digest_Should_Return_Whatever_Value_It_Was_Wrapped_Around()
    {
        for seed in Sample_Seeds()
        {
            let digest = Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);

            assert_eq!(InputDigest::From_Digest(digest).Digest(), digest);
        }
    }

    /// A handful of distinct byte fills, so the round trip is shown for more than one value.
    fn Sample_Seeds() -> [u8; 3]
    {
        return [0, ARBITRARY_SEED_BYTE, MAX_SEED_BYTE];
    }
}
