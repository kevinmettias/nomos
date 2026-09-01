//! Band 25 — the one provider of `nomos.cap.naming.policy` (`nomos-cap-naming-policy`).
//!
//! # Why this is `crates/repository/`, not `crates/languages/`
//!
//! Every existing band-25 crate is a `nomos-lang-*` provider: it reads one language's
//! source or manifest format. This provider reads `standards.json` — one file, declared
//! once for the whole repository, applicable across every language a repository's source
//! is written in. Naming it a language provider, or placing it under `crates/languages/`,
//! would misstate what it does the same way `nomos-workspace` living under
//! `crates/languages/` would. `OD-RULES-011` names this the first crate under a new
//! `crates/repository/` top-level directory, grouped by what kind of input a crate reads
//! rather than by band alone, the same reason `crates/languages/` exists apart from
//! `crates/capabilities/`.
//!
//! # Why this provider is a crate of its own, unlike a second offer inside another
//!
//! The identical reason `nomos-lang-rust-deny` is its own crate rather than a second offer
//! inside another provider: this provider's answer comes from reading a file this
//! workspace's own text already holds, through a caller-supplied
//! [`nomos_platform::FileSystem`], the same port every filesystem-reading provider in this
//! workspace runs through — so this crate depends on `nomos-platform` and not on any
//! concrete implementation of it; the composition root chooses that.
//!
//! # What `OD-RULES-011` this satisfies
//!
//! `OD-RULES-011` decided a rule's configurable parameters are read as a capability fact
//! through the `FactReader` seam `OD-RULES-010` already established. This crate is that
//! decision's first provider: [`reading::Discover_Workspace`] is the reader;
//! [`provider::Materialize_Workspace`] turns its answer into the one fact this capability's
//! `IncrementalGranularity::WholeWorkspace` ceiling allows. Composing this crate into a
//! real check run, and the rules that read what it materializes, are `nomos-check-
//! orchestration` and `nomos-rules`' own territory, not this crate's — this decision's own
//! next increments.

#![forbid(unsafe_code)]

mod guarantee;
#[path = "fact_context.rs"]
mod provider;
mod reading;

pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{Discover_Workspace, NamingPolicyError};
