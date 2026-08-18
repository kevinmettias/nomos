//! Band 24 -- the language-agnostic core of an installable-unit manifest.
//!
//! `OD-PACKAGE-007` split this out of `nomos-lang-package`: everything `PKG-007`'s four
//! version domains name that does not name a specific language. `PackageVersion`,
//! `ProtocolRange` and `ProviderRegistration` were already language-agnostic on
//! inspection -- none names Rust or imports a provider crate. `language_versions` stays
//! unresolved, raw labels rather than a typed enum, because there is no one typed shape a
//! version label takes across languages.
//!
//! [`Parse_Manifest`] and [`Read_Manifest`] take a `known_providers` allowlist as a
//! parameter rather than a constant, so this crate depends on nothing above
//! `nomos-contracts`, `serde` and `serde_json` -- never on a specific language's provider
//! crates. A language-specific package crate (`nomos-lang-package` for Rust, and any
//! future sibling) depends on this crate directly, supplies its own known-providers list
//! and resolves `language_versions`' raw labels against whatever typed domain its own
//! ecosystem actually needs.

#![forbid(unsafe_code)]

mod manifest;
mod protocol_range;
mod provider_registration;
mod reader;
mod version;

pub use manifest::PackageManifest;
pub use protocol_range::ProtocolRange;
pub use provider_registration::ProviderRegistration;
pub use reader::{ManifestError, Parse_Manifest, Read_Manifest};
pub use version::PackageVersion;
