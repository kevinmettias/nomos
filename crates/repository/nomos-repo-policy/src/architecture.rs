//! The one provider of `nomos.cap.architecture.declaration` (`nomos-cap-architecture`).
//!
//! # Why this reads its own file and its five siblings do not
//!
//! The five beside it read `standards.json`, each under a key `code-standards` already owns
//! and decodes. This one cannot: that file is shared and decoded with unknown fields
//! disallowed, so a key this repository would own outright is not available in it, and the key
//! that looks closest is the one `OD-RULES-029`'s amendment forbids by name. `reading`'s own
//! doc carries the measurement and the two alternatives it rules out.
//!
//! # Why this is a module of [`crate`] and not a crate of its own
//!
//! The identical reason [`crate::naming`] is: `OD-PACKAGE-015` measured a provider against
//! independent versioning, an enforced isolation boundary, and a real consumer wanting it
//! without its siblings, and found none of the three true. This module is not an
//! `OD-RULES-011` family -- `OD-RULES-029` decided that, and [`nomos_cap_architecture`]'s own
//! doc says why -- but the packaging question `OD-PACKAGE-015` asked is about the provider,
//! not about what the capability carries, and its answer is unchanged here.

#[path = "architecture/fact_context.rs"]
mod provider;
mod guarantee;
mod reading;

pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{ArchitectureError, Discover_Workspace, ARCHITECTURE_JSON};
