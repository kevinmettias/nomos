//! A rule's own registration contract, extracted before a second rule needs it.
//!
//! `OD-RULES-004` decided this now, ahead of a second rule shipping, so a rule package can
//! be designed against a stated shape rather than by copying `Check_Completeness_Mirrors`'s
//! own hand-written composition into a crate that does not exist yet. Registration and
//! selection are different questions — `OD-HOST-004` already answered the second one, and
//! this module answers only the first: how a rule becomes a nameable thing a registry
//! holds, not which rules `Run()` calls. Nothing here is consulted by `Run()`, and nothing
//! here changes what runs on any given `nomos check`.

mod rule_offer;
mod rule_registry;
mod rule_registry_error;

pub use rule_offer::RuleOffer;
pub use rule_registry::RuleRegistry;
pub use rule_registry_error::RuleRegistryError;
