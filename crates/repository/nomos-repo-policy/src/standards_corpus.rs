//! The one provider of `nomos.cap.standards.corpus` (`nomos-cap-standards-corpus`).
//!
//! # Why this reads its own file and the policy siblings do not
//!
//! The `OD-RULES-011` families read `standards.json`, each under a key `code-standards`
//! already owns and decodes. This one cannot, for the identical reason
//! [`crate::test_material`] and [`crate::architecture`] cannot: that file is shared and decoded
//! with unknown fields disallowed, and a key nomos would own outright is not available in it.
//! `OD-HOST-009` decided where a repository-declared criterion that is not one of
//! `code-standards`' own keys travels — a dedicated, language-neutral file at the repository
//! root, the convention `nomos-gate.json`, `nomos-architecture.json` and
//! `nomos-test-material.json` already set. `reading`'s own doc carries the convention and this
//! provider's own file name.
//!
//! # Why this is a module of [`crate`], not a crate of its own
//!
//! The identical reason [`crate::naming`] is: `OD-PACKAGE-015` measured a provider against
//! independent versioning, an enforced isolation boundary, and a real consumer wanting it
//! without its siblings, and found none of the three true. What is new here is the *reader*
//! rather than the packaging question, and the packaging answer is unchanged by it.
//!
//! # What is deliberately not a seventh policy family
//!
//! `OD-RULES-029` decided that a family is a repository's policy parameters, and
//! [`crate::architecture`] is already the exception that rule is written with: a description a
//! rule judges against rather than a parameter one is told by. A standards corpus is the same
//! shape one layer out — a population of declared rules the repository owns, which a judgment
//! reads — so this module sits beside `architecture` and `test_material` rather than in the
//! family the `standards.json` walk serves. Nothing here reads `standards.json`, and no
//! `ScopedBlock` reaches it.

#[path = "standards_corpus/fact_context.rs"]
mod provider;
mod guarantee;
mod reading;

pub use guarantee::{Declared_Guarantee, Provider_Offer, PROVIDER};
pub use provider::{FactContext, Materialize_Workspace, PolicyFact};
pub use reading::{Discover_Workspace, StandardsCorpusError, STANDARDS_CORPUS_JSON};
