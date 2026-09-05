//! The one provider of `nomos.cap.limits.policy` (`nomos-cap-limits-policy`).
//!
//! # Why this reads `standards.json`, not one language's own source
//!
//! The identical reason [`crate::naming`] does: this provider reads `standards.json` — one
//! file, declared once for the whole repository — not one language's source or manifest
//! format.
//!
//! # Why this is a module of [`crate`], not a crate of its own
//!
//! The identical reason [`crate::naming`] is: `OD-PACKAGE-015` measured this provider
//! against independent versioning, an enforced isolation boundary, and a real consumer
//! wanting it without its four siblings, and found none of the three true. The capability
//! and the `ProviderId` `OD-RULES-011` built are unchanged; only the packaging is.
//!
//! # What `OD-RULES-011` this satisfies
//!
//! `OD-RULES-011` named a threshold family (the file-size triggers) as its own future
//! instance of the naming decision. This module is that instance's provider:
//! [`reading::Discover_Workspace`] is the reader, reading a `limits` block this item is the
//! first to declare in `standards.json` (with values equal to `nomos-rules`' current
//! hardcoded file-size defaults, so nothing regresses); [`provider::Materialize_Workspace`]
//! turns its answer into the one fact this capability's `IncrementalGranularity::
//! WholeWorkspace` ceiling allows. Composing this module into a real check run, and the
//! rules that read what it materializes, are `nomos-check-orchestration` and `nomos-rules`'
//! own territory, not this module's.

#[path = "limits/limits_policy_fact_production.rs"]
mod determinism;
mod guarantee;
#[path = "limits/fact_context.rs"]
mod provider;
mod reading;

pub use determinism::LimitsPolicyFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{Discover_Workspace, LimitsPolicyError};
