//! The language-agnostic manifest shape.

use crate::{PackageVersion, ProtocolRange, ProviderRegistration};
use nomos_contracts::{PackageId, PackageKind};

/// An installable-unit manifest, minus anything a specific language would supply.
///
/// `OD-PACKAGE-007`'s split of `nomos-lang-package`: identity, [`PackageKind`], and
/// `PKG-007`'s four version domains, with the third domain (`language_versions`) left as
/// raw, unresolved labels rather than a typed enum. There is no one typed shape a
/// version label takes across languages -- Rust's four editions, a semver string, a
/// single integer, a date-stamped release are all real shapes a language's package might
/// need -- so resolving a label to a typed value is left to whichever language-specific
/// crate wraps this one, the same way `nomos-lang-package` resolves each label against
/// [`nomos_contracts`]-external `RustEdition`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageManifest
{
    /// This package's identity.
    pub package_id: PackageId,
    /// Which kind of package this is. [`crate::Parse_Manifest`] refuses every value other
    /// than [`PackageKind::LanguagePackage`] before a value is ever constructed; the
    /// field is carried anyway so a caller holding one of several package kinds behind a
    /// common representation can still ask which this is without downcasting.
    pub package_kind: PackageKind,
    /// This package's own release. `PKG-007`'s first version domain.
    pub package_version: PackageVersion,
    /// The Nomos protocol versions this package's providers speak. `PKG-007`'s second
    /// version domain, and never read from the same field as `package_version`.
    pub protocol_range: ProtocolRange,
    /// The language versions this package recognizes, as the manifest spelled them.
    /// `PKG-007`'s third version domain, unresolved: this crate does not know what a
    /// valid label looks like for any language, only that a manifest naming none of them
    /// recognizes nothing. Never empty.
    pub language_versions: Vec<String>,
    /// The providers this package registers, each carrying its own tool version --
    /// `PKG-007`'s fourth version domain. Never empty, for the same reason as
    /// `language_versions`.
    pub providers: Vec<ProviderRegistration>,
}
