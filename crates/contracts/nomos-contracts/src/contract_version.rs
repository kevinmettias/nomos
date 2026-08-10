//! How a schema or contract says whether an existing consumer still understands it.

use serde::{Deserialize, Serialize};

/// A schema or contract version.
///
/// Two components rather than three: a contract either stays compatible with existing
/// consumers or it does not, and a patch level invites the belief that a third kind of
/// change exists which needs no thought.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContractVersion
{
    /// Incremented when a consumer written against the previous version breaks.
    pub major: u16,
    /// Incremented for additions a previous consumer can ignore.
    pub minor: u16,
}

impl ContractVersion
{
    /// Constructs a version.
    #[must_use]
    pub const fn New(major: u16, minor: u16) -> Self
    {
        return Self { major, minor };
    }

    /// Whether a consumer written against `self` can read data produced at `other`.
    #[must_use]
    pub const fn Can_Read(self, other: Self) -> bool
    {
        return self.major == other.major && self.minor >= other.minor;
    }
}

impl core::fmt::Display for ContractVersion
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "v{}.{}", self.major, self.minor);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Version_Should_Read_Its_Own_And_Older_Minors_Only()
    {
        let reader = ContractVersion::New(1, 3);

        assert!(reader.Can_Read(ContractVersion::New(1, 3)));
        assert!(reader.Can_Read(ContractVersion::New(1, 0)));
        assert!(!reader.Can_Read(ContractVersion::New(1, 4)));
        assert!(!reader.Can_Read(ContractVersion::New(2, 0)));
        assert!(!reader.Can_Read(ContractVersion::New(0, 9)));
    }
}
