//! Zone: Provider — the first `IntegrationPackage` manifest maturity, and the first consumer
//! of `PackageKind::IntegrationPackage` anywhere in this workspace.
//!
//! `OD-PACKAGE-003` decided what an `IntegrationPackage` is: a peer's connection to Nomos,
//! never the content that crosses it. Its "Declaring A Placement Is Not Performing One"
//! section is the shape this crate implements -- a manifest **declares a materialization
//! intent** (a source, a repository-relative target path, the ownership class
//! `OD-PACKAGE-004` gives that target, and the publication scope `OD-PACKAGE-005` gives it)
//! and performs no write. [`IntegrationPackage`] is that manifest; [`MaterializationIntent`]
//! is one declared placement; [`OwnershipClass`] and [`PublicationScope`] are the two
//! records' vocabularies, spelled exactly as they name them.
//!
//! # What is deliberately not here
//!
//! **No write, copy, stage or rollback logic.** Atomicity, each ownership class's conflict
//! handling, staging and rollback are mechanics that know nothing about a peer connection,
//! and `OD-PACKAGE-003` routes them to a generic, mechanism-owned materializer consuming an
//! intent no matter which package kind declared it. No such materializer exists in this
//! workspace; when one is built it is named as Nomos's own per `D-138` rather than staged
//! under a platform name, and this crate hands it a declaration rather than teaching itself
//! filesystem mechanics. That is also why [`OwnershipClass`] and [`PublicationScope`] live
//! here beside the manifest rather than in a crate of their own: `OD-CAPABILITY-002`'s rule
//! is that a contract earns a crate when a second party names it, and today the only party
//! to either enum is this manifest. The day a materializer needs them, both move below this
//! crate, and nothing in their shape presumes otherwise.
//!
//! **No typed surface axis.** `OD-PACKAGE-003` leaves open whether its placement table's
//! rows (agent contract file, per-agent adapter, skills, hooks, MCP registration, CI
//! projections, connector configuration) should become an `IntegrationSurfaceKind`, and says
//! it is a question for whoever builds the second `IntegrationPackage`. This is the first,
//! so a surface is a free label carried as text, and no enum closes the vocabulary.
//!
//! **No declared-only rows.** `OD-PACKAGE-003`'s table also names surfaces a package records
//! without placing. Those are not materialization intents, and this maturity carries only
//! intents.
//!
//! # Its place among the package crates
//!
//! A deliberate peer of `nomos-lang-rust-package`, `nomos-lang-go-package`,
//! `nomos-model-package`, `nomos-rule-package` and `nomos-tool-package`, not a dependent of
//! any of them: all wrap `nomos-package`'s generic core (`OD-PACKAGE-007`) for one
//! `PackageKind` family, and `OD-PACKAGE-015`'s criterion is met the same way each sibling
//! meets it -- an installable unit earns its own crate. This crate hand-rolls its own reader
//! over `nomos-package`'s plain shared types (`ProtocolRange`, `PackageVersion`) rather than
//! calling `nomos_package::Parse_Manifest`, the pattern every non-`LanguagePackage` sibling
//! already uses: that shared entry point refuses every `PackageKind` other than
//! `LanguagePackage`, and its `language_versions` and `providers` domains are meaningless for
//! a package that registers no provider and recognizes no language. `PKG-007`'s first two
//! version domains are genuinely kind-agnostic, so [`PackageVersion`] and [`ProtocolRange`]
//! are re-exported from `nomos_package` unchanged rather than retyped.
//!
//! Built ahead of a consumer under `OD-ROADMAP-001`'s licence: no installer reads this
//! manifest yet, and the crate's own tests stand in for one.

#![forbid(unsafe_code)]

mod integration_package;
mod materialization_intent;
mod ownership_class;
mod publication_scope;
mod reader;
mod target;

pub use integration_package::IntegrationPackage;
pub use materialization_intent::MaterializationIntent;
pub use nomos_package::{PackageVersion, ProtocolRange};
pub use ownership_class::OwnershipClass;
pub use publication_scope::PublicationScope;
pub use reader::{ManifestError, Parse_Manifest, Read_Manifest, SCHEMA_VERSION};
