//! The first `ToolProvider` manifest maturity, and the first consumer of
//! `PackageKind::ToolProvider` anywhere in this workspace.
//!
//! `PackageKind::ToolProvider` was, before this crate, the one populated `PackageKind`
//! with no manifest crate: `LanguagePackage` has `nomos-lang-rust-package` and
//! `nomos-lang-go-package`, `RulePackage` has `nomos-rule-package`,
//! `ModelBackendPackage`/`AgentExecutorPackage` share `nomos-model-package`. Two real
//! `ToolProvider`s exist today -- `nomos-lang-rust-clippy` (`nomos.lang.rust.clippy`) and
//! `nomos-lang-rust-deny` (`nomos.lang.rust.deny`) -- and neither had a manifest kind that
//! could declare it. `P47-TOOLPROVIDER-HAS-NO-PACKAGE-2` closes that gap.
//!
//! `OD-CAPABILITY-013` decided this workspace's shape for it: a closed, twelve-name FAMILY
//! vocabulary reused verbatim from `toolspec`, classifying a provider's own registration
//! rather than anything a rule names. [`Family`] is that vocabulary; [`ToolProviderRegistration`]
//! is a registration that carries it -- the axis this crate cares about, rather than
//! whether a provider launches a subprocess, which `OD-CAPABILITY-013` reserves as a
//! separate, unattempted axis. `nomos-lang-rust-clippy` classifies as [`Family::Linter`]
//! and `nomos-lang-rust-deny` as [`Family::PackageManager`] -- a dependency-graph auditor
//! beside `nomos-lang-rust-cargo`'s dependency-graph reader, not a source-code rule engine
//! beside clippy.
//!
//! A deliberate peer of `nomos-lang-rust-package`, `nomos-lang-go-package`,
//! `nomos-model-package` and `nomos-rule-package`, not a dependent of any of them: all wrap
//! `nomos-package`'s generic core for one `PackageKind` family. This crate hand-rolls its
//! own reader over `nomos-package`'s plain shared types
//! (`ProtocolRange`, `PackageVersion`) rather than calling `nomos_package::Parse_Manifest`,
//! the same pattern `nomos-rule-package` and `nomos-model-package` already use: that shared
//! entry point refuses every `PackageKind` other than `LanguagePackage`, and its
//! `language_versions` domain is meaningless for a `ToolProvider` manifest, which carries
//! no language version at all. This crate re-exports `nomos_package::{PackageVersion,
//! ProtocolRange}` unchanged, because `PKG-007`'s first two version domains are genuinely
//! kind-agnostic, the same reason `nomos-rule-package` and `nomos-model-package` do.

#![forbid(unsafe_code)]

mod family;
mod known_providers;
mod reader;
mod tool_package;
mod tool_provider_registration;

pub use family::Family;
pub use known_providers::{Is_Known, KNOWN_PROVIDERS};
pub use nomos_package::{PackageVersion, ProtocolRange};
pub use reader::{ManifestError, Parse_Manifest, Read_Manifest, SCHEMA_VERSION};
pub use tool_package::ToolPackage;
pub use tool_provider_registration::ToolProviderRegistration;
