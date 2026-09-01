//! Band 25 — the one provider of `nomos.cap.scripting.policy` (`nomos-cap-scripting-policy`).
//!
//! # Why this is `crates/repository/`, not `crates/languages/`
//!
//! The identical reason `nomos-repo-standards` and `nomos-repo-limits` sit here rather
//! than under `crates/languages/`: this provider reads `standards.json` — one file,
//! declared once for the whole repository — not one language's source or manifest format.
//!
//! # Why this provider is a crate of its own, unlike a second offer inside another
//!
//! The identical reason its two siblings are each their own crate: this provider's answer
//! comes from reading a file this workspace's own text already holds, through a
//! caller-supplied [`nomos_platform::FileSystem`], the same port every filesystem-reading
//! provider in this workspace runs through.
//!
//! # What `OD-RULES-011` this satisfies
//!
//! A third instance of `OD-RULES-011`'s decision, after naming and numeric limits: this
//! crate's [`reading::Discover_Workspace`] is the reader; [`provider::Materialize_
//! Workspace`] turns its answer into the one fact this capability's `IncrementalGranularity
//! ::WholeWorkspace` ceiling allows. Composing this crate into a real check run, and the
//! rule that reads what it materializes, are `nomos-check-orchestration` and `nomos-rules`'
//! own territory, not this crate's — this decision's own next increments.

#![forbid(unsafe_code)]

#[path = "scripting_policy_fact_production.rs"]
mod determinism;
mod guarantee;
#[path = "fact_context.rs"]
mod provider;
mod reading;

pub use determinism::ScriptingPolicyFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{Discover_Workspace, ScriptingPolicyError};
