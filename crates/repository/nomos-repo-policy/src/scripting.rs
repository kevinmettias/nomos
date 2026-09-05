//! The one provider of `nomos.cap.scripting.policy` (`nomos-cap-scripting-policy`).
//!
//! # Why this reads `standards.json`, not one language's own source
//!
//! The identical reason [`crate::naming`] and [`crate::limits`] do: this provider reads
//! `standards.json` — one file, declared once for the whole repository — not one
//! language's source or manifest format.
//!
//! # Why this is a module of [`crate`], not a crate of its own
//!
//! The identical reason its siblings are: `OD-PACKAGE-015` measured this provider against
//! independent versioning, an enforced isolation boundary, and a real consumer wanting it
//! without its four siblings, and found none of the three true. The capability and the
//! `ProviderId` `OD-RULES-011` built are unchanged; only the packaging is.
//!
//! # What `OD-RULES-011` this satisfies
//!
//! A third instance of `OD-RULES-011`'s decision, after naming and numeric limits: this
//! module's [`reading::Discover_Workspace`] is the reader; [`provider::Materialize_
//! Workspace`] turns its answer into the one fact this capability's `IncrementalGranularity
//! ::WholeWorkspace` ceiling allows. Composing this module into a real check run, and the
//! rule that reads what it materializes, are `nomos-check-orchestration` and `nomos-rules`'
//! own territory, not this module's.

#[path = "scripting/scripting_policy_fact_production.rs"]
mod determinism;
mod guarantee;
#[path = "scripting/fact_context.rs"]
mod provider;
mod reading;

pub use determinism::ScriptingPolicyFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{Discover_Workspace, ScriptingPolicyError};
