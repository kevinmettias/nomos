//! The range of Nomos protocol (contract) versions a package's providers speak.
//!
//! `PKG-007`'s second version domain. Distinct from [`crate::PackageVersion`]: a package
//! can be re-released -- a documentation fix, a new provider added -- without the
//! protocol compatibility of its providers changing at all, and PKG-007 forbids folding
//! the two together.

use nomos_contracts::ContractVersion;
use serde::{Deserialize, Serialize};

/// The inclusive range of [`ContractVersion`]s this package's providers offer against.
///
/// Expressed in `nomos_contracts::ContractVersion` rather than a bespoke type, because
/// that is what a provider's offer already carries -- reusing it means a package's claim
/// can be checked against what its providers actually declare, rather than against a
/// second spelling of the same idea that could drift from the first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolRange
{
    /// The oldest contract version a provider in this package may still offer against.
    pub minimum: ContractVersion,
    /// The newest.
    pub maximum: ContractVersion,
}

impl ProtocolRange
{
    /// Constructs a range. Does not validate ordering; see [`Self::Is_Ordered`].
    #[must_use]
    pub const fn New(minimum: ContractVersion, maximum: ContractVersion) -> Self
    {
        return Self { minimum, maximum };
    }

    /// Whether `minimum` sorts at or before `maximum`.
    ///
    /// `ContractVersion`'s own `Ord` is major then minor, which is exactly the order a
    /// range's two ends have to agree with -- a `minimum` that reads newer than
    /// `maximum` names an empty range, and nothing downstream of this type should have
    /// to notice that on its own.
    #[must_use]
    pub const fn Is_Ordered(&self) -> bool
    {
        return self.minimum.major < self.maximum.major
            || (self.minimum.major == self.maximum.major && self.minimum.minor <= self.maximum.minor);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_A_Single_Version_Range_Is_Ordered()
    {
        let range = ProtocolRange::New(ContractVersion::New(1, 0), ContractVersion::New(1, 0));

        assert!(range.Is_Ordered());
    }

    #[test]
    fn Test_A_Widening_Range_Is_Ordered()
    {
        let range = ProtocolRange::New(ContractVersion::New(1, 0), ContractVersion::New(1, 3));

        assert!(range.Is_Ordered());
    }

    #[test]
    fn Test_An_Inverted_Range_Is_Not_Ordered()
    {
        let range = ProtocolRange::New(ContractVersion::New(1, 3), ContractVersion::New(1, 0));

        assert!(!range.Is_Ordered());
    }
}
