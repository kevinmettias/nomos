//! Band 25 — the one provider of `nomos.cap.goals.policy` (`nomos-cap-goals-policy`).
//!
//! # Why this is `crates/repository/`, not `crates/languages/`
//!
//! The identical reason `nomos-repo-standards`, `nomos-repo-limits`, `nomos-repo-scripting`
//! and `nomos-repo-words` sit here rather than under `crates/languages/`: this provider
//! reads `standards.json` — one file, declared once for the whole repository — not one
//! language's source or manifest format. It is the strongest case of the four: a goal is a
//! statement about the system, and a system does not have one set of purposes per language.
//!
//! # Why this provider is a crate of its own, unlike a second offer inside another
//!
//! The identical reason its four siblings are each their own crate: this provider's answer
//! comes from reading a file this workspace's own text already holds, through a
//! caller-supplied [`nomos_platform::FileSystem`], the same port every filesystem-reading
//! provider in this workspace runs through.
//!
//! # Which `OD-RULES-011` this satisfies
//!
//! A fifth instance of `OD-RULES-011`'s decision, after naming, numeric limits, scripting
//! and words: this crate's [`reading::Discover_Workspace`] is the reader;
//! [`provider::Materialize_Workspace`] turns its answer into the one fact this capability's
//! `IncrementalGranularity::WholeWorkspace` ceiling allows. Composing this crate into a real
//! check run, and the rule that reads what it materializes, are `nomos-check-orchestration`
//! and `nomos-rules`' own territory, not this crate's — this decision's own next increments.
//!
//! # This repository declares no goals, and that is the reader's most-exercised path
//!
//! Unlike scripting and words, whose blocks `standards.json` already carried before anyone
//! read them, this workspace declares no `goals` at all. That is not a gap to fill here:
//! goal traceability is a discipline a system takes on, and inventing a goal set for this
//! repository would be this provider deciding something only the repository can decide. So
//! the reader's "declares nothing" path is the one this workspace actually exercises, and
//! its tests say so rather than reading around it.

#![forbid(unsafe_code)]

#[path = "goals_policy_fact_production.rs"]
mod determinism;
mod guarantee;
#[path = "fact_context.rs"]
mod provider;
mod reading;

pub use determinism::GoalsPolicyFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{Discover_Workspace, GoalsPolicyError};
