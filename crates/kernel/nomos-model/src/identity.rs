//! Identity: what makes two observations the same thing.
//!
//! [`CompositeIdentity`] is the answer for a *declaration*, and the three types beside it are
//! its components: the shape that survives a rename, where the declaration came from, and how
//! two declarations that legitimately share a name are told apart. Each is a separate file
//! because each is separately replaceable — a language package may state a different
//! [`IdentityPolicy`] without touching what an identity is made of.
//!
//! # Two kinds of thing are identified here, and they do not share components
//!
//! [`FindingOccurrenceId`] answers the same question for a *finding occurrence*, and it is not a
//! `CompositeIdentity` in a different hat. A declaration has a language, a qualified name, a
//! signature and a structural shape; a finding has none of those, and a mapping between the two
//! would be invented rather than measured. They live in one module because they answer one
//! question — when are these two observations the same thing — and are kept apart because the
//! material that answers it differs entirely.
//!
//! # Identity and transition are also not the same thing
//!
//! An identity names a thing **within one pinned observation**.
//! [`IdentityTransitionKind`] names a **relation between identities across snapshots** — whether
//! this is the same thing renamed, moved, split, recreated, or something nothing can resolve.
//! A reader asking whether a finding persisted or recurred is asking for a transition, not for
//! an identity, and `OD-GATE-030` names the run history that question additionally needs.
//!
//! # A finding has two identities, and asking the wrong one gets a confident wrong answer
//!
//! [`FindingOccurrenceId`] is scoped to one pinned result and takes the finding's locations,
//! because two violations of one rule in one file differ by nothing else. [`OccurrenceLineageId`]
//! is the identity that survives a revision instead, and pays for it by giving up exactly that
//! discrimination: it takes the rule, the subject and the rule's own sentence, so two occurrences
//! described alike are one lineage. Neither is the other under a different name, and each one's
//! own doc states what it costs.
//!
//! [`OccurrenceTally`] is where a lineage meets [`IdentityTransitionKind`]: given how many of a
//! record's states observed one lineage and which, it answers with `ExactContinuity`, `Recreated`
//! or nothing at all. It lives here rather than in whichever service keeps the record, so that a
//! second `Persistent`/`Reintroduced`/`Moved` vocabulary is never minted beside this one.

// How an identity moves from one form to another, beneath the identity it is of.
mod transition;
mod transition_kind;

pub use transition::Transition as IdentityTransition;
pub use transition_kind::TransitionKind as IdentityTransitionKind;

mod composite_identity;
mod policy;
mod source_provenance;
mod structural_fingerprint;

pub use composite_identity::CompositeIdentity;
pub use policy::Policy as IdentityPolicy;
pub use source_provenance::SourceProvenance;
pub use structural_fingerprint::StructuralFingerprint;

// The identity of one finding occurrence, and the guard that keeps its injectivity a checked
// property rather than an assumed one.
mod finding_occurrence_id;
mod occurrence_collision;

pub use finding_occurrence_id::FindingOccurrenceId;
pub use occurrence_collision::{Occurrence_Collisions_In, OccurrenceCollision};

// The identity of a recurring violation across revisions, and what a record of the states in
// between establishes about one.
mod occurrence_lineage_id;
mod occurrence_tally;

pub use occurrence_lineage_id::OccurrenceLineageId;
pub use occurrence_tally::OccurrenceTally;
