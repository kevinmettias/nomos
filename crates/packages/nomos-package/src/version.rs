//! A package's own release identity.
//!
//! One of `PKG-007`'s four version domains -- package version -- kept as its own type so
//! it cannot be mistaken for [`nomos_contracts::ContractVersion`] (the protocol range
//! domain) at a call site that happens to typecheck. The two answer different questions:
//! a `PackageVersion` says which release of the package this is, a `ContractVersion` says
//! which protocol revisions a consumer of it can read.

use serde::{Deserialize, Serialize};

/// A package's own version: major, minor, patch.
///
/// Three components, deliberately unlike `ContractVersion`'s two. A contract's version
/// only has to say whether it stays compatible with an existing consumer, so two
/// components are enough; a package release is a broader claim, and this type is not
/// only used for a package's own version -- [`crate::ProviderRegistration`]
/// reuses it for a registered provider's tool version, which is `PKG-007`'s fourth
/// domain and a different question again, carried in a different field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PackageVersion
{
    /// Incremented on a breaking release.
    pub major: u16,
    /// Incremented on a compatible addition.
    pub minor: u16,
    /// Incremented on a fix that changes neither.
    pub patch: u16,
}

impl PackageVersion
{
    /// Constructs a version.
    #[must_use]
    pub const fn New(major: u16, minor: u16, patch: u16) -> Self
    {
        return Self { major, minor, patch };
    }
}

impl core::fmt::Display for PackageVersion
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Display_Should_Render_Three_Dotted_Components()
    {
        assert_eq!(PackageVersion::New(1, 2, 3).to_string(), "1.2.3");
    }

    /// `PackageVersion` and `nomos_contracts::ContractVersion` are different types, so a
    /// caller cannot pass one where the other is expected -- the compile-time half of
    /// keeping PKG-007's domains distinct. There is no runtime assertion for a type
    /// error; this test exists so the claim is written down beside the type rather than
    /// only in the module doc.
    #[test]
    fn Test_Ordering_Is_Lexicographic_By_Major_Then_Minor_Then_Patch()
    {
        assert!(PackageVersion::New(1, 0, 0) < PackageVersion::New(1, 0, 1));
        assert!(PackageVersion::New(1, 0, 9) < PackageVersion::New(1, 1, 0));
        assert!(PackageVersion::New(1, 9, 9) < PackageVersion::New(2, 0, 0));
    }
}
