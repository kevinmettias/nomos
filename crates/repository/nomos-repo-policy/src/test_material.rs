//! The one provider of `nomos.cap.test.material.policy` (`nomos-cap-test-material-policy`).
//!
//! # Why this reads its own file and its five policy siblings do not
//!
//! The five `OD-RULES-011` families read `standards.json`, each under a key `code-standards`
//! already owns and decodes. This one cannot, for the identical reason
//! [`crate::architecture`] cannot: that file is shared and decoded with unknown fields
//! disallowed, and a key nomos would own outright is not available in it. `OD-HOST-009`
//! decided where a repository-declared criterion that is not one of `code-standards`' own
//! keys travels: a dedicated, language-neutral file at the repository root, the convention
//! `nomos-gate.json` and `nomos-architecture.json` already set. `reading`'s own doc carries
//! the convention.
//!
//! # Why this is a module of [`crate`], not a crate of its own
//!
//! The identical reason [`crate::naming`] is: `OD-PACKAGE-015` measured a provider against
//! independent versioning, an enforced isolation boundary, and a real consumer wanting it
//! without its siblings, and found none of the three true. This module is a sixth
//! `OD-RULES-011` family — a repository's policy parameters — but the packaging question
//! `OD-PACKAGE-015` asked is about the provider, not about what the capability carries, and
//! its answer is unchanged here.

#[path = "test_material/fact_context.rs"]
mod provider;
mod guarantee;
mod reading;

pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{Discover_Workspace, TestMaterialPolicyError, TEST_MATERIAL_JSON};
