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

    /// The version the reader under test was built against.
    const READER_VERSION: ContractVersion = ContractVersion::New(1, 3);
    /// One minor ahead of what `READER_VERSION` understands.
    const A_NEWER_MINOR: u16 = 4;
    /// A major line other than the reader's, at its first minor.
    const A_DIFFERENT_MAJOR: u16 = 2;
    /// A minor within a major line older than the reader's.
    const AN_OLDER_MAJOR_MINOR: u16 = 9;

    /// The major and minor `Test_New_Should_Construct_A_Version_From_Its_Major_And_Minor` builds from.
    const CONSTRUCTED_MAJOR: u16 = 4;
    const CONSTRUCTED_MINOR: u16 = 2;

    #[test]
    fn Test_Can_Read_Should_Accept_Its_Own_And_Older_Minors_Only()
    {
        let reader = READER_VERSION;

        assert!(reader.Can_Read(READER_VERSION));
        assert!(reader.Can_Read(ContractVersion::New(1, 0)));
        assert!(!reader.Can_Read(ContractVersion::New(1, A_NEWER_MINOR)));
        assert!(!reader.Can_Read(ContractVersion::New(A_DIFFERENT_MAJOR, 0)));
        assert!(!reader.Can_Read(ContractVersion::New(0, AN_OLDER_MAJOR_MINOR)));
    }

    #[test]
    fn Test_New_Should_Construct_A_Version_From_Its_Major_And_Minor()
    {
        let version = ContractVersion::New(CONSTRUCTED_MAJOR, CONSTRUCTED_MINOR);

        assert_eq!(version.major, CONSTRUCTED_MAJOR);
        assert_eq!(version.minor, CONSTRUCTED_MINOR);
    }
}
