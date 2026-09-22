//! Zone: Provider — the C# `LanguagePackage` manifest format, and the third consumer of
//! `nomos-package`'s language-agnostic manifest core.
//!
//! # What a third consumer settles
//!
//! `OD-PACKAGE-007` split `nomos-package` out of `nomos-lang-package` so a second language's
//! package crate could depend on it directly, never on the Rust-specific wrapper.
//! `nomos-lang-go-package` was that second consumer and proved the split usable. What it could
//! not prove is that the split is *general*, because Go's version domain turned out to have the
//! same shape as the generic field it resolves: two integers and a dot, every time.
//!
//! C#'s does not. `LangVersion` is a major with an *optional* minor — `7.3` and `12` are both
//! whole, well-formed C# language versions — so [`CsharpVersion`] is the first version domain in
//! this workspace where the label's own shape varies. That is the case the generic
//! `language_versions: Vec<String>` field exists for, and this crate is the first one to use it
//! for something a `{major, minor}` pair could not have carried.
//!
//! # The shape
//!
//! [`LanguagePackage`] carries a [`nomos_contracts::PackageId`], a
//! [`nomos_contracts::PackageKind`] pinned to `LanguagePackage`, and `PKG-007`'s four version
//! domains — [`PackageVersion`], [`ProtocolRange`], a list of [`CsharpVersion`] and a list of
//! [`ProviderRegistration`] — the identical shape `nomos_lang_package::LanguagePackage` and
//! `nomos_lang_go_package::LanguagePackage` carry for their own languages.
//!
//! [`Read_Manifest`] and [`Parse_Manifest`] resolve that shape from JSON, refusing — with a
//! named [`ManifestError`] variant, never a silent default — a missing field, a `package_kind`
//! other than `LanguagePackage`, a version domain that will not parse, or a provider this
//! package's crate does not know.

#![forbid(unsafe_code)]

mod csharp_version;
mod known_providers;
mod language_package;
mod reader;

pub use csharp_version::CsharpVersion;
pub use known_providers::{Is_Known, KNOWN_PROVIDERS};
pub use language_package::LanguagePackage;
pub use nomos_package::{PackageVersion, ProtocolRange, ProviderRegistration};
pub use reader::{ManifestError, Parse_Manifest, Read_Manifest};
