//! The first `LanguagePackage` manifest shape.

use crate::{PackageVersion, ProtocolRange, ProviderRegistration, RustEdition};
use nomos_contracts::{PackageId, PackageKind};

/// A `LanguagePackage`: registration and translation boundary for one language
/// ecosystem, per `ARCH-001`.
///
/// This is the *first* manifest maturity, scoped to what `OD-PACKAGE-001` asked for and
/// no further: identity, [`PackageKind`], `PKG-007`'s four version domains, and which
/// providers this package registers. `PKG-022`'s larger field list -- recognition, file
/// mappings, capability claims, canonical mappings, environment requirements,
/// projections, quality objectives, conformance suites -- is a later manifest maturity
/// `OD-PACKAGE-001` explicitly did not scope into this first step, and none of it is
/// attempted here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguagePackage
{
    /// This package's identity.
    pub package_id: PackageId,
    /// Always [`PackageKind::LanguagePackage`] for a value that a
    /// [`crate::Parse_Manifest`] produced -- the reader refuses every other kind before
    /// a `LanguagePackage` is ever constructed. The field is carried anyway, because a
    /// caller holding one of several package kinds behind a common representation still
    /// needs to be able to ask which this is without downcasting.
    pub package_kind: PackageKind,
    /// This package's own release. `PKG-007`'s first version domain.
    pub package_version: PackageVersion,
    /// The Nomos protocol versions this package's providers speak. `PKG-007`'s second
    /// version domain, and never read from the same field as `package_version`.
    pub protocol_range: ProtocolRange,
    /// The Rust editions this package recognizes. `PKG-007`'s third version domain.
    /// Never empty: a `LanguagePackage` that recognizes no language version recognizes
    /// nothing.
    pub language_versions: Vec<RustEdition>,
    /// The providers this package registers, each carrying its own tool version --
    /// `PKG-007`'s fourth version domain. Never empty, for the same reason as
    /// `language_versions`: `OD-PACKAGE-001` named registering `nomos-lang-rust` and
    /// `nomos-lang-rust-scan` as this manifest's whole reason for existing.
    pub providers: Vec<ProviderRegistration>,
}
