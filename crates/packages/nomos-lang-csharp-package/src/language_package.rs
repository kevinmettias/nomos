//! The first C# `LanguagePackage` manifest shape.

use crate::CsharpVersion;
use nomos_contracts::{PackageId, PackageKind};
use nomos_package::{PackageVersion, ProtocolRange, ProviderRegistration};

/// A `LanguagePackage`: registration and translation boundary for the C# ecosystem, per
/// `ARCH-001` — the identical shape `nomos_lang_package::LanguagePackage` and
/// `nomos_lang_go_package::LanguagePackage` carry, scoped to the same first manifest maturity
/// `OD-PACKAGE-001` asked for and no further.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguagePackage
{
    /// This package's identity.
    pub package_id: PackageId,
    /// Always [`PackageKind::LanguagePackage`] for a value that a [`crate::Parse_Manifest`]
    /// produced.
    pub package_kind: PackageKind,
    /// This package's own release. `PKG-007`'s first version domain.
    pub package_version: PackageVersion,
    /// The Nomos protocol versions this package's providers speak. `PKG-007`'s second version
    /// domain.
    pub protocol_range: ProtocolRange,
    /// The C# language versions this package recognizes. `PKG-007`'s third version domain. Never
    /// empty: a `LanguagePackage` that recognizes no language version recognizes nothing.
    pub language_versions: Vec<CsharpVersion>,
    /// The providers this package registers, each carrying its own tool version — `PKG-007`'s
    /// fourth version domain. Never empty, for the same reason as `language_versions`.
    pub providers: Vec<ProviderRegistration>,
}
