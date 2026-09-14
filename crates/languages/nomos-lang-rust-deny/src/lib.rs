//! Zone: Provider — the one provider of `nomos.cap.dependency.policy` (`nomos-cap-dependency-policy`).
//!
//! # Why this provider is a crate of its own, unlike a second offer inside another
//!
//! The identical reason `nomos-lang-rust-clippy` is its own crate rather than a second
//! offer inside `nomos-lang-rust-cargo`: this provider's answer is `cargo deny`'s own
//! analysis over the fully resolved dependency graph, not a fact this workspace's own text
//! already holds, so answering it means running `cargo deny` and reading its own JSON
//! stream. The process itself runs through a caller-supplied
//! [`nomos_platform::ProcessLauncher`], the same port every subprocess-backed provider in
//! this workspace runs through — so this crate depends on `nomos-platform` and not on any
//! concrete implementation of it; the composition root chooses that.
//!
//! # Why `advisories` is not among the checks this provider runs
//!
//! `cargo deny check` accepts four checks: `advisories`, `bans`, `licenses`, `sources`.
//! Only `advisories` fetches the `RustSec` advisory database over the network — this
//! workspace's own `.github/workflows/gate.yml` already draws that exact line between the
//! three that read only `Cargo.lock` and this crate's own `deny.toml`, and the one that
//! does not. A provider whose own guarantee claims a deterministic, no-network answer
//! cannot honestly run a check that silently depends on what a remote database currently
//! contains — `advisories` is a distinct future provider's own question, not this one's.
//!
//! # What `OD-RULES-010` this satisfies
//!
//! `OD-RULES-010` decided the first real `ToolProvider`'s output is a fact a native rule
//! judges. This crate is that decision's second real instance: [`reading::Discover_Workspace`]
//! is the reader; [`provider::Materialize_Workspace`] turns its answer into the one fact
//! this capability's `IncrementalGranularity::WholeWorkspace` ceiling allows. Composing
//! this crate into a real check run, and the rule that reads what it materializes, are
//! `nomos-check-orchestration` and `nomos-rules`' own territory, not this crate's.

#![forbid(unsafe_code)]

#[path = "dependency_policy_fact_production.rs"]
mod determinism;
mod guarantee;
#[path = "fact_context.rs"]
mod provider;
#[path = "deny_error.rs"]
mod reading;

pub use determinism::DependencyPolicyFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{DenyError, Discover_Workspace};
