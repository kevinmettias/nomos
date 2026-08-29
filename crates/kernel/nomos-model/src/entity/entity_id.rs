use nomos_contracts::Digest128;
use serde::{Deserialize, Serialize};

/// Identity of a canonical entity, independent of any snapshot.
///
/// Answers "which thing is this" across time. Pair it with a snapshot to get a
/// [`crate::SnapshotEntity`], which answers "which thing is this, as it was then".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityId(Digest128);

impl EntityId
{
    /// Wraps a digest as an entity identity.
    #[must_use]
    pub const fn From_Digest(digest: Digest128) -> Self
    {
        return Self(digest);
    }

    /// The underlying digest.
    #[must_use]
    pub const fn Digest(&self) -> Digest128
    {
        return self.0;
    }
}

impl core::fmt::Display for EntityId
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
    use crate::Content_Digest;

    #[test]
    fn Test_Entity_Id_Should_Render_As_Its_Digest()
    {
        let digest = Content_Digest(b"symbol");
        let id = EntityId::From_Digest(digest);

        assert_eq!(id.to_string(), digest.to_string());
    }
}
