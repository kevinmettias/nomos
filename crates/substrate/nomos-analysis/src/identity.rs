use nomos_contracts::Digest128;
use nomos_model::Digest_Of_Parts;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InputDigest(Digest128);

impl InputDigest
{
    #[must_use]
    pub fn Of(parts: &[&[u8]]) -> Self
    {
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
