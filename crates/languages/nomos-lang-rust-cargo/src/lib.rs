//! Band 25 — the one provider of `nomos.cap.dependency.edges` (`nomos-cap-dependency`).
//!
//! # Why this provider is a crate of its own, unlike a second offer inside `nomos-lang-rust`
//!
//! `nomos-lang-rust` and `nomos-lang-rust-scan` are both pure functions over bytes a
//! caller already read: `Materialize(subject, text, context)`, with no filesystem access
//! and no subprocess. This provider cannot be that shape — a first-party dependency edge
//! is Cargo's own resolution of a manifest, not a fact this workspace's text already
//! holds, so answering it means running `cargo metadata` and reading its JSON. That is a
//! real difference in what a provider *is*, not a style choice, and it is why this
//! provider lives in its own crate rather than as a second offer inside `nomos-lang-rust`.
//!
//! # What `OD-RULES-003` this satisfies
//!
//! `OD-RULES-003` named the shape this crate now fills: the observed dependency graph is
//! a fact a capability provider establishes at [`nomos_contracts::FactVariant::SemanticallyResolved`],
//! not a rule reading `Cargo.toml` for itself. [`Discover_Workspace`] is the reader;
//! [`Materialize_Workspace`] turns its answer into facts. Composing this crate into a real
//! check run is separate, later work — see below.
//!
//! # What this crate does not do
//!
//! It is not composed into `nomos-check-orchestration::Run` and does not change what
//! `nomos check` does. `nomos-check-orchestration` performs no I/O of its own by design —
//! every existing provider it calls is a pure function over bytes the composition root
//! already read — and wiring this provider in means touching where that root walks a
//! tree, `nomos-cli`, which a concurrent item held live at the time this crate was
//! written. This crate is real and tested standing alone, the same "additive and unwired"
//! shape `nomos_rules::RuleRegistry` carried from `OD-RULES-004` until something consumed
//! it.

#![forbid(unsafe_code)]

mod determinism;
mod guarantee;
mod metadata;
mod provider;

pub use determinism::DependencyFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use metadata::{DiscoveredPackage, Discover_Workspace, MetadataError};
pub use provider::{FactContext, Materialize_Workspace, PackageFact};
