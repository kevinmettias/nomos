//! The first `IntegrationPackage` manifest shape.

use nomos_materialization::MaterializationIntent;
use nomos_contracts::{PackageId, PackageKind};
use nomos_package::{PackageVersion, ProtocolRange};

/// An `IntegrationPackage` manifest, at its first maturity.
///
/// `OD-PACKAGE-003`'s own subject: a peer's connection to Nomos, declared as a list of
/// placements and never performed here. Carries identity, `PackageKind` (restricted to
/// [`PackageKind::IntegrationPackage`]), `PKG-007`'s first two version domains reused
/// unchanged from `nomos-package`, and the materialization intents this package declares
/// in place of `nomos-package`'s own `language_versions` and `providers` domains -- an
/// `IntegrationPackage` recognizes no language and registers no provider, so neither domain
/// has anything to say about it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegrationPackage
{
    /// This package's identity.
    pub package_id: PackageId,
    /// Always [`PackageKind::IntegrationPackage`] for a value [`crate::Parse_Manifest`]
    /// produced -- the reader refuses every other kind before a value is constructed.
    pub package_kind: PackageKind,
    /// This package's own release. `PKG-007`'s first version domain.
    pub package_version: PackageVersion,
    /// The Nomos protocol versions this package's connection speaks. `PKG-007`'s second
    /// version domain.
    pub protocol_range: ProtocolRange,
    /// The placements this package declares, each one a source, a repository-relative
    /// target, an ownership class and a publication scope. Never empty: an
    /// `IntegrationPackage` declaring no placement materializes nothing.
    pub intents: Vec<MaterializationIntent>,
}
