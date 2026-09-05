//! The one provider of `nomos.cap.naming.policy` (`nomos-cap-naming-policy`).
//!
//! # Why this reads `standards.json`, not one language's own source
//!
//! This provider reads `standards.json` — one file, declared once for the whole
//! repository, applicable across every language a repository's source is written in. It is
//! not a `nomos-lang-*`-shaped provider for that reason: naming it a language provider
//! would misstate what it does. `OD-RULES-011` grouped it under `crates/repository/`,
//! parallel to `crates/languages/`, for the identical reason `crates/languages/` exists
//! apart from `crates/capabilities/` — by what kind of input is read, not by band alone.
//!
//! # Why this is a module of [`crate`], not a crate of its own
//!
//! `OD-RULES-011` and `OD-RULES-019` gave this provider its own crate by the same
//! precedent every `nomos-lang-*` provider follows. `OD-PACKAGE-015` measured that
//! precedent against a sharper test — independent versioning, an enforced isolation
//! boundary, or a real consumer wanting this provider without its four siblings — and
//! found none of the three true here: every consumer that depends on this provider depends
//! on all five together, every time. This module carries that finding out. The capability,
//! the `ProviderId`, and the registration `OD-RULES-011` built are unchanged; only the
//! packaging is.
//!
//! # What `OD-RULES-011` this satisfies
//!
//! `OD-RULES-011` decided a rule's configurable parameters are read as a capability fact
//! through the `FactReader` seam `OD-RULES-010` already established. This module is that
//! decision's first provider: [`reading::Discover_Workspace`] is the reader;
//! [`provider::Materialize_Workspace`] turns its answer into the one fact this capability's
//! `IncrementalGranularity::WholeWorkspace` ceiling allows. Composing this module into a
//! real check run, and the rules that read what it materializes, are `nomos-check-
//! orchestration` and `nomos-rules`' own territory, not this module's.

#[path = "naming/naming_policy_fact_production.rs"]
mod determinism;
mod guarantee;
#[path = "naming/fact_context.rs"]
mod provider;
mod reading;

pub use determinism::NamingPolicyFactProduction;
pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{Discover_Workspace, NamingPolicyError};
