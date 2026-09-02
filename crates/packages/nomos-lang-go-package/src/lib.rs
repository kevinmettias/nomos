//! Band 26 -- the first Go `LanguagePackage` manifest format, and the first real second
//! consumer of `nomos-package`'s language-agnostic manifest core.
//!
//! # What this closes
//!
//! `OD-PACKAGE-007` split `nomos-package` (band 24) out of `nomos-lang-package`
//! specifically so a second language's package crate could depend on it directly, never on
//! the Rust-specific wrapper. `OD-PACKAGE-006`'s amendment retired waiting for a real
//! second language before trusting that split; `nomos-lang-go` (`P14-LANG-GO-SYNTAX-
//! PROVIDER`) is that real second language, and this crate is the consumer the split was
//! built for, proven against a real language rather than left a hypothesis.
//!
//! # The shape
//!
//! [`LanguagePackage`] carries a [`nomos_contracts::PackageId`], a
//! [`nomos_contracts::PackageKind`] pinned to `LanguagePackage`, and `PKG-007`'s four
//! version domains -- [`PackageVersion`], [`ProtocolRange`], a list of [`GoVersion`] and a
//! list of [`ProviderRegistration`] -- the identical shape
//! `nomos_lang_package::LanguagePackage` carries for Rust. [`GoVersion`] is a genuinely
//! different typed domain from `RustEdition`, not a renamed copy of it -- see its own
//! module doc for why an open-ended `{major, minor}` pair is the honest shape here where a
//! small, closed enum was the honest shape for Rust's four editions.
//!
//! [`Read_Manifest`] and [`Parse_Manifest`] resolve that shape from JSON, refusing -- with
//! a named [`ManifestError`] variant, never a silent default -- a missing field, a
//! `package_kind` other than `LanguagePackage`, a version domain that will not parse, or a
//! provider this package's crate does not know.
//!
//! `nomos-lang-go-modules` (`nomos.cap.dependency.edges`) is deliberately not a provider
//! this crate can register -- see `known_providers`'s own module doc for why a
//! `LanguagePackage`'s scope is the language's syntax provider and not every capability
//! provider that happens to read its ecosystem's files.

#![forbid(unsafe_code)]

mod go_version;
mod known_providers;
mod language_package;
mod reader;

pub use go_version::GoVersion;
pub use known_providers::{Is_Known, KNOWN_PROVIDERS};
pub use language_package::LanguagePackage;
pub use nomos_package::{PackageVersion, ProtocolRange, ProviderRegistration};
pub use reader::{ManifestError, Parse_Manifest, Read_Manifest};
