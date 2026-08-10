use nomos_contracts::Digest128;
use serde::{Deserialize, Serialize};

/// A hash of a symbol's shape, independent of its name and location.
///
/// The component that survives a rename. Without it, renaming a function looks like
/// deleting one and adding another, every suppression attached to it is orphaned, and
/// its history restarts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StructuralFingerprint(Digest128);

impl StructuralFingerprint
{
    /// Wraps a digest as a fingerprint.
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
