//! Zone: Substrate — the canonical model kernel.
//!
//! What a thing *is*, how it stays the same thing across time, and how two claims on
//! overlapping things are compared.
//!
//! # The subject model
//!
//! Three canonical entity kinds — [`Artifact`], [`Symbol`], [`Resource`] — and one
//! addressable reference to them, [`Subject`]. The reference *references*; it does not
//! duplicate. A subject that carried its own copy of a symbol's identity would be a
//! second place for that identity to be wrong.
//!
//! [`SnapshotEntity`] is the occurrence of an entity under one snapshot, build variant
//! and configuration. Findings, architecture nodes, feature members and test paths all
//! address a `SnapshotEntity` rather than a bare entity, because "the same function"
//! compiled two ways is one thing to talk about and two things to measure.
//!
//! # Identity is never a path and a line
//!
//! [`CompositeIdentity`] is built from language, provider-native identity, qualified
//! name, signature and a structural fingerprint. This is the most consequential lesson
//! carried from the prototype: with path-and-line identity, every comparative feature —
//! diffing, history, trajectories, co-change, transformation tracking — is a join with
//! no key, and a reformatting commit looks like the whole file was rewritten.
//!
//! When identity does change, [`Transition`] records *how* and with what confidence,
//! rather than presenting a rename as a deletion plus an unrelated arrival.
//!
//! [`FindingOccurrenceId`] does take a finding's locations, and that is not this rule being
//! bent. The rule governs *what a claim is attributed to* — a finding's subject stays a digest,
//! deliberately coarse, and stays what a suppression and a baseline match on, so a tolerated
//! entry survives an edit above it. The occurrence identity answers the question after that one:
//! given the subject, which occurrence within it. Two violations of one rule in one file differ
//! by nothing else, so the location is the only occurrence-discriminating material there is. It
//! is scoped to one pinned result, never used to match a tolerance across revisions, and
//! [`Occurrence_Collisions_In`] keeps its injectivity a checked property rather than a hope.
//!
//! [`OccurrenceLineageId`] is what a tolerance across revisions matches on instead, and it is a
//! different identity rather than the same one relaxed: rule, subject and the rule's own
//! sentence, with the locations left out so that a reformatting commit does not expire it. What
//! it gives up for that is discrimination — two occurrences described alike are one lineage —
//! and [`OccurrenceTally`] is what turns a record of a lineage's sightings into the transition
//! vocabulary above, so the answer to *persisted or recurred* is never re-invented downstream.
//!
//! # One exclusion primitive
//!
//! [`SubjectSet`] and [`Intersection`] live here rather than beside the correction
//! scheduler that was originally going to own them, because three separate subsystems
//! need to ask the same question — *do these two pieces of work touch the same things?*
//! — and they sit at three different levels: the work ledger coordinating agents
//! building Nomos, the correction scheduler planning parallel waves, and agent leases
//! within a run. A type defined at any one of those levels cannot be used by the others.
//!
//! Defining it once, here, is what makes "one exclusion primitive" a structural fact
//! rather than a convention three implementations are each asked to remember.
//!
//! [`Subject_Of_Path`] is here for the same reason and arrived the same way: the ledger,
//! the composition root and the integration corpus had each written the identical rule for
//! reducing a path spelling to a subject. `OD-MODEL-001` records why the spelling rule is
//! one rule while the ledger's record-identifier fold stays with the ledger.

#![forbid(unsafe_code)]

mod confidence;
mod digest;
mod entity;
mod evidence;
mod identity;
mod path;
mod subject;
mod transition;

pub use digest::{Content_Digest, Digest_Of_Parts};
pub use confidence::Confidence;
pub use entity::{Artifact, ArtifactKind, EntityId, Resource, ResourceKind, SnapshotEntity, Symbol, SymbolKind};
pub use evidence::{Coverage, CoverageGap, Evidence, EvidenceRef};
pub use identity::{CompositeIdentity, FindingOccurrenceId, IdentityPolicy, IdentityTransition, IdentityTransitionKind, Occurrence_Collisions_In, OccurrenceCollision, OccurrenceLineageId, OccurrenceTally, SourceProvenance, StructuralFingerprint};
pub use path::{Normalize_Path, Subject_Of_Path};
pub use subject::{Intersection, SetResolution, Subject, SubjectKind, SubjectSet, SubjectTarget, UnknownReason};
pub use transition::Transition;
