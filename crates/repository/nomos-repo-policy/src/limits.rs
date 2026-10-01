//! The one provider of `nomos.cap.limits.policy` (`nomos-cap-limits-policy`).
//!
//! # Why this reads `nomos-limits.json`, not `standards.json` and not one language's source
//!
//! It reads one file, declared once for the whole repository, rather than a language's source
//! or manifest format -- the reason [`crate::naming`] gives too. But the file is its own.
//! This provider used to read a `limits` block from `standards.json`, a block no repository
//! that also runs code-standards could write, because that tool decodes the file strictly and
//! names no such block. `OD-RULES-035` moved the family to `nomos-limits.json`; `reading`'s own
//! doc carries the measurement and the file's shape.
//!
//! # Why this is a module of [`crate`], not a crate of its own
//!
//! The identical reason [`crate::naming`] is: `OD-PACKAGE-015` measured this provider
//! against independent versioning, an enforced isolation boundary, and a real consumer
//! wanting it without its siblings, and found none of the three true. The capability and the
//! `ProviderId` `OD-RULES-011` built are unchanged; only the packaging is.
//!
//! # What `OD-RULES-011` this satisfies
//!
//! `OD-RULES-011` named a threshold family (the file-size triggers) as its own future
//! instance of the naming decision. This module is that instance's provider:
//! [`reading::Discover_Workspace`] is the reader; [`provider::Materialize_Workspace`] turns
//! its answer into the one fact this capability's `IncrementalGranularity::WholeWorkspace`
//! ceiling allows. Composing this module into a real check run, and the rules that read what
//! it materializes, are `nomos-check-orchestration` and `nomos-rules`' own territory, not
//! this module's.

#[path = "limits/limits_policy_fact_production.rs"]
mod determinism;
mod guarantee;
#[path = "limits/fact_context.rs"]
mod provider;
mod reading;

pub use determinism::LimitsPolicyFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{Discover_Workspace, LimitsPolicyError, LIMITS_JSON};
