//! Band 26 -- the first `LanguagePackage` manifest format, and the first consumer of
//! `nomos_contracts::PackageId` and `PackageKind` outside their own crate.
//!
//! # What this closes
//!
//! `docs/records/OD-PACKAGE-001-independently-versioned-is-a-property-of-a-package-and-this-build-has-no-package.md`
//! measured `PackageKind` and `PackageId` as declared and consumed nowhere, and found no
//! package manifest on disk anywhere in this workspace -- not narrower than that, since
//! `ARCH-001` and `ARCH-002` describe a declaration, not a Rust API, and neither
//! `nomos-lang-rust` (a `ToolProvider` in the corpus's own sense) nor `nomos-rules`
//! (holding one rule's judgment with no manifest of its own) is the artifact those
//! requirements name.
//!
//! The record laid out four steps and named the first as items 1 and 2 together: *"An
//! installable unit exists that is not a Cargo crate ... Something reads it and refuses
//! what it cannot resolve ... This is the step that gives `PackageKind` and `PackageId`
//! their first consumer, and it must land with the first manifest rather than before
//! it."* This crate is that step, with `nomos-lang-rust` and `nomos-lang-rust-scan` as
//! the natural first subject the record names -- a `LanguagePackage` that registers both
//! as providers.
//!
//! Steps 3 (a rule citing its contract version) and 4 (the `publish` question, which
//! `OD-PACKAGE-001` already settles) are out of scope here. So is `PKG-022`'s larger
//! field list -- recognition, file/project mappings, capability claims, canonical
//! mappings, environment requirements, projections, quality objectives, conformance
//! suites -- which is a later manifest maturity the record explicitly did not scope into
//! this first step.
//!
//! # The shape
//!
//! [`LanguagePackage`] carries a [`nomos_contracts::PackageId`], a
//! [`nomos_contracts::PackageKind`] pinned to `LanguagePackage`, and `PKG-007`'s four
//! version domains as four distinct fields that cannot be collapsed into one:
//! [`PackageVersion`] (the package's own release), [`ProtocolRange`] (the Nomos protocol
//! versions its providers speak), a list of [`RustEdition`] (the language versions it
//! recognizes) and a list of [`ProviderRegistration`] (which providers it registers, and
//! each one's own tool version). [`Read_Manifest`] and [`Parse_Manifest`] resolve that
//! shape from JSON, refusing -- with a named [`ManifestError`] variant, never a silent
//! default -- a missing field, a `package_kind` other than `LanguagePackage`, a version
//! domain that will not parse, or a provider this package's crate does not know.
//!
//! `nomos-contracts` gains no consumer from this: `PackageId` and `PackageKind` are read
//! here, never constructed for it to write back, and no code in that crate changes.
//!
//! # What moved to `nomos-package`
//!
//! `OD-PACKAGE-007` split `PackageVersion`, `ProtocolRange`, `ProviderRegistration` and
//! `ManifestError` into `nomos-package`, the language-agnostic core this crate wraps:
//! this crate never depended on Rust in those four types, only in `RustEdition` and
//! `KNOWN_PROVIDERS`, and a second language's package crate can now depend on
//! `nomos-package` directly without this crate, or Rust's two providers, in the way.
//! This crate's own public surface -- `LanguagePackage`, `Parse_Manifest`,
//! `Read_Manifest`, `RustEdition`, `KNOWN_PROVIDERS`, `Is_Known` -- is unchanged.

#![forbid(unsafe_code)]

mod known_providers;
mod language_version;
mod manifest;
mod reader;

pub use known_providers::{Is_Known, KNOWN_PROVIDERS};
pub use language_version::RustEdition;
pub use manifest::LanguagePackage;
pub use nomos_package::{PackageVersion, ProtocolRange, ProviderRegistration};
pub use reader::{ManifestError, Parse_Manifest, Read_Manifest};
