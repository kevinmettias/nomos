//! Band 26 -- the first `ModelBackendPackage`/`AgentExecutorPackage` manifest maturity,
//! and the first consumer of those two `PackageKind` variants anywhere in this workspace.
//!
//! `OD-PACKAGE-010` draws the boundary this crate builds inside: identity, `PackageKind`
//! (restricted to `ModelBackendPackage` and `AgentExecutorPackage`), `PKG-007`'s first two
//! version domains reused unchanged from `nomos-package`, and a new [`ModelSelection`]
//! domain answering `MODEL-ROUTE-037`'s opening clause -- a versioned discovered model
//! catalog, or an explicit declaration that selection is opaque or executor-controlled.
//! `MODEL-ROUTE-037`'s own catalog-entry detail and `MODEL-ROUTE-038`..`049`'s
//! routing/conformance system are a later manifest maturity, named in that record and not
//! attempted here -- the same split `OD-PACKAGE-001` already drew between `LanguagePackage`'s
//! first step and `PKG-022`'s larger field list.
//!
//! A deliberate peer of `nomos-lang-package`, not a dependent of it: both wrap
//! `nomos-package`'s generic core for one `PackageKind` family, and neither depends on the
//! other. This crate re-exports [`nomos_package::PackageVersion`] and
//! [`nomos_package::ProtocolRange`] unchanged rather than redefining them, because
//! `PKG-007`'s first two version domains are genuinely kind-agnostic; it does not reuse
//! `nomos_package::PackageManifest`, `Parse_Manifest` or `ProviderRegistration`, because
//! their fourth domain (`language_versions`/`providers`) is shaped for a language's
//! capability providers, which a model backend does not register.

#![forbid(unsafe_code)]

mod manifest;
mod model_selection;
mod reader;

pub use manifest::ModelRoutePackage;
pub use model_selection::ModelSelection;
pub use nomos_package::{PackageVersion, ProtocolRange};
pub use reader::{ManifestError, Parse_Manifest, Read_Manifest, SCHEMA_VERSION};
