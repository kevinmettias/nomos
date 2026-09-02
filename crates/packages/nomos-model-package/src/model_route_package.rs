//! The first `ModelBackendPackage`/`AgentExecutorPackage` manifest shape.

use crate::ModelSelection;
use nomos_contracts::{PackageId, PackageKind};
use nomos_package::{PackageVersion, ProtocolRange};

/// A `ModelBackendPackage` or `AgentExecutorPackage`, at its first manifest maturity.
///
/// `OD-PACKAGE-010` draws this boundary: identity, [`PackageKind`] (restricted to the two
/// kinds this crate reads), `PKG-007`'s first two version domains unchanged from
/// `nomos-package`, and [`ModelSelection`] in place of the fourth domain -- a model backend
/// does not register `nomos_capability` providers the way a `LanguagePackage` does, so
/// `nomos-package`'s `providers` field does not carry over. `MODEL-ROUTE-037`'s
/// catalog-entry detail and `MODEL-ROUTE-038`..`049`'s routing/conformance system are a
/// later manifest maturity, not attempted here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRoutePackage
{
    /// This package's identity.
    pub package_id: PackageId,
    /// Always [`PackageKind::ModelBackendPackage`] or [`PackageKind::AgentExecutorPackage`]
    /// for a value [`crate::Parse_Manifest`] produced -- the reader refuses every other
    /// kind before a value is ever constructed. Carried anyway, the same reason
    /// `nomos_package::PackageManifest::package_kind` is: a caller holding one of several
    /// package kinds behind a common representation still needs to ask which this is.
    pub package_kind: PackageKind,
    /// This package's own release. `PKG-007`'s first version domain.
    pub package_version: PackageVersion,
    /// The Nomos protocol versions this package speaks. `PKG-007`'s second version
    /// domain, and never read from the same field as `package_version`.
    pub protocol_range: ProtocolRange,
    /// How this package exposes which model or executor a request reaches.
    /// `MODEL-ROUTE-037`'s opening clause, and `PKG-007`'s fourth version domain's real
    /// shape for this package kind.
    pub model_selection: ModelSelection,
}
