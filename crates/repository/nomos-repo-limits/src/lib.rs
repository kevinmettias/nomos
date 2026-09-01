//! Band 25 — the one provider of `nomos.cap.limits.policy` (`nomos-cap-limits-policy`).
//!
//! # Why this is `crates/repository/`, not `crates/languages/`
//!
//! The identical reason `nomos-repo-standards` sits here rather than under
//! `crates/languages/`: this provider reads `standards.json` — one file, declared once for
//! the whole repository, applicable across every language a repository's source is written
//! in — not one language's source or manifest format.
//!
//! # Why this provider is a crate of its own, unlike a second offer inside another
//!
//! The identical reason `nomos-repo-standards` is its own crate rather than folding a
//! second capability into it: `nomos-12`'s own stated principle for keeping the sibling
//! naming and limits capabilities apart — "no shared JSON-reading abstraction for two
//! instances" — extends to their providers the same way. This provider's answer comes from
//! reading a file this workspace's own text already holds, through a caller-supplied
//! [`nomos_platform::FileSystem`], the same port every filesystem-reading provider in this
//! workspace runs through.
//!
//! # What `OD-RULES-011` this satisfies
//!
//! `OD-RULES-011` named a threshold family (the file-size triggers) as its own future
//! instance of the naming decision. This crate is that instance's provider:
//! [`reading::Discover_Workspace`] is the reader, reading a `limits` block this item is the
//! first to declare in `standards.json` (with values equal to `nomos-rules`' current
//! hardcoded file-size defaults, so nothing regresses); [`provider::Materialize_Workspace`]
//! turns its answer into the one fact this capability's `IncrementalGranularity::
//! WholeWorkspace` ceiling allows. Composing this crate into a real check run, and the
//! rules that read what it materializes, are `nomos-check-orchestration` and `nomos-rules`'
//! own territory, not this crate's — this decision's own next increments.

#![forbid(unsafe_code)]

#[path = "limits_policy_fact_production.rs"]
mod determinism;
mod guarantee;
#[path = "fact_context.rs"]
mod provider;
mod reading;

pub use determinism::LimitsPolicyFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{Discover_Workspace, LimitsPolicyError};
