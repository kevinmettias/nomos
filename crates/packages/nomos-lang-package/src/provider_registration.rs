//! One provider a `LanguagePackage` registers, and the tool version behind it.

use crate::PackageVersion;
use nomos_contracts::ProviderId;

/// A registered provider: which one, and what version of the underlying tool answers on
/// its behalf.
///
/// `tool_version` is `PKG-007`'s fourth domain, and it is carried here rather than as a
/// single scalar on the manifest because "the provider version" is not a question a
/// package with more than one registered provider can answer with one number -- it is
/// meaningless until it says which provider it is about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderRegistration
{
    /// The provider this registers.
    pub provider: ProviderId,
    /// The version of the tool behind that provider. For `nomos-lang-rust` and
    /// `nomos-lang-rust-scan` today, this is that crate's own `Cargo.toml` `version` --
    /// which `OD-PACKAGE-001` is explicit is not the package version, the protocol
    /// range, or a language version, but is exactly this domain: what build of the tool
    /// is answering.
    pub tool_version: PackageVersion,
}
