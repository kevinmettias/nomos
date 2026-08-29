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
//! # Composed into `nomos-check-orchestration::Run`
//!
//! `crate::facts::Materialize_Dependencies` calls [`Materialize_Workspace`] over the tree a
//! check run was asked about and writes what it returns into the same store the syntax
//! rules read, so `nomos check` judges dependency direction as part of an ordinary run.
//! This is the one provider in that composition with I/O of its own — every other provider
//! it calls is a pure function over bytes the composition root already read — which is why
//! `Materialize_Dependencies` takes the walked tree's own root path rather than only the
//! sources that walk already produced.
//!
//! The process itself runs through a caller-supplied [`nomos_platform::ProcessLauncher`],
//! not `std::process::Command` directly: [`Materialize_Workspace`] and [`Discover_Workspace`]
//! are generic over it, so this crate depends on `nomos-platform` and not on any concrete
//! implementation of that port. Two composition roots reach `nomos-check-orchestration::Run`
//! today, `nomos-cli` directly and `nomos-api` through `nomos_gate_orchestration::Run_Gate`,
//! and both choose `StdProcessLauncher`.

#![forbid(unsafe_code)]

#[path = "dependency_fact_production.rs"]
mod determinism;
mod guarantee;
#[path = "metadata_error.rs"]
mod metadata;
#[path = "fact_context.rs"]
mod provider;

pub use determinism::DependencyFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use metadata::{DiscoveredPackage, Discover_Workspace, MetadataError};
pub use provider::{FactContext, Materialize_Workspace, PackageFact};
