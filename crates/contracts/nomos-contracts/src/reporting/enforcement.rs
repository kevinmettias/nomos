//! Whether the thing that claims to enforce a rule can actually fail a build.
//!
//! Adopted from the sibling xvpe workspace, which invented the distinction and proved
//! it against a real corpus. The idea is one sentence long:
//!
//! > Naming an enforcer is a claim about **existence**. Whether that enforcer runs is a
//! > claim about **execution**. Only the second one can be checked against reality, and
//! > it is the one that matters.
//!
//! Both prototypes measured what happens without it. In the Nomos prototype, 747 of 860
//! standards declared `enforced_by: [review]` — 87% of the law was a wish — and fifty
//! landed documents named a tool that did not exist anywhere, found only by reading them
//! all by hand. In xvpe, six rules currently declare an enforcer that no workflow
//! invokes, including *both* rules that define its strategy-surface admission bar.
//!
//! The point is not that unenforced rules are bad. It is that an unenforced rule and an
//! enforced one must not look the same, because a rule that claims a check it does not
//! have teaches every reader to disbelieve the rest of the corpus.
//!
//! The four types are the two halves and the two ways they come apart:
//! [`EnforcerRef`] is what a rule names, [`GateCategory`] is what naming it amounts to,
//! [`EnforcementBreach`] is a way the naming is false, and [`EnforcementReach`] is where
//! the claim and the reality are held side by side without either overwriting the other.

mod enforcement_breach;
mod enforcer_ref;
mod gate_category;
mod enforcement_reach;

pub use enforcement_breach::EnforcementBreach;
pub use enforcer_ref::EnforcerRef;
pub use gate_category::GateCategory;
pub use enforcement_reach::EnforcementReach;
