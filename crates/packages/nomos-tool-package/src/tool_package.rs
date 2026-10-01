//! The first `ToolProvider` manifest shape.

use crate::tool_provider_registration::ToolProviderRegistration;
use nomos_contracts::{PackageId, PackageKind};
use nomos_package::{PackageVersion, ProtocolRange};

/// A `ToolProvider` manifest, at its first maturity.
///
/// `P47-TOOLPROVIDER-HAS-NO-PACKAGE-2`'s own subject: `PackageKind::ToolProvider` is the
/// one populated `PackageKind` with no manifest crate before this one. Carries identity,
/// `PackageKind` (restricted to [`PackageKind::ToolProvider`]), `PKG-007`'s first two
/// version domains reused unchanged from `nomos-package`, and the providers this package
/// registers -- each carrying its own tool version and `OD-CAPABILITY-013` FAMILY
/// classification in place of `nomos-package`'s own `language_versions` domain, which is a
/// `LanguagePackage`-specific concept a `ToolProvider` manifest has no use for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolPackage
{
    /// This package's identity.
    pub package_id: PackageId,
    /// Always [`PackageKind::ToolProvider`] for a value [`crate::Parse_Manifest`]
    /// produced -- the reader refuses every other kind before a value is constructed.
    pub package_kind: PackageKind,
    /// This package's own release. `PKG-007`'s first version domain.
    pub package_version: PackageVersion,
    /// The Nomos protocol versions this package's providers speak. `PKG-007`'s second
    /// version domain.
    pub protocol_range: ProtocolRange,
    /// The `ToolProvider`s this package registers, each carrying its own tool version and
    /// FAMILY classification. Never empty: a `ToolProvider` manifest registering no
    /// provider registers nothing.
    pub providers: Vec<ToolProviderRegistration>,
}
