//! Zone: Substrate — the generic materializer `OD-PACKAGE-003` routed to, and the
//! vocabulary a declaring package and this mechanism both name.
//!
//! `OD-PACKAGE-003`'s "Declaring A Placement Is Not Performing One" decided that a package
//! declares a materialization intent and never performs the write, and that atomicity, the
//! conflict handling each ownership class requires, staging and rollback belong to "a
//! generic materializer, mechanism-owned, consuming a materialization intent no matter
//! which package kind declared it". [`Materialize`] is that mechanism.
//!
//! # The three behaviours
//!
//! `OD-PACKAGE-004`'s three classes, each performed here rather than described:
//!
//! - [`OwnershipClass::UserOwned`] is never written, under any circumstance. The whole run
//!   is refused before a single byte is placed, because "refuses the write outright" and
//!   "writes the other three targets first" are not the same promise.
//! - [`OwnershipClass::GeneratedOwned`] is overwritten unconditionally, whether the target
//!   was there or not and whatever it held.
//! - [`OwnershipClass::Composed`] has only its declared owned region written. Everything
//!   outside that region — the free region a person authored — survives byte for byte, and
//!   every case where the mechanism cannot prove that survives is a refusal rather than a
//!   best guess. [`Refusal`] names them one by one.
//!
//! # Atomicity
//!
//! Per target, by the port: `nomos_platform::FileSystem::Replace_Atomically` promises a
//! temporary file and a rename, so no reader ever sees a half-written target and a failure
//! leaves the previous bytes intact.
//!
//! Across a multi-target intent, by this crate: every target's prior contents are read
//! before any of them is written, and a write that fails partway through undoes every write
//! already made, in reverse order — restoring what was there, or removing what was not.
//! A run that fails leaves the tree as it found it, and says which of
//! [`MaterializationError::RolledBack`] or [`MaterializationError::RollbackFailed`] it
//! achieved rather than claiming the first and hoping.
//!
//! # Publication scope gates nothing here
//!
//! [`PublicationScope`] is carried on every intent and reported on every [`Placement`], and
//! it decides no write. `OD-PACKAGE-005` makes it a property of *where an asset may travel*
//! — whether it may enter repository-distributed state — and states the guard it owes as a
//! guard on that transition, not on the placement. A mechanism that refused to write a
//! `Local` target would be enforcing a publication policy at the wrong boundary and would
//! make the two axes the same axis, which that record's own controls table refuses.
//!
//! # What is deliberately not here
//!
//! **No package kind, crate, rule, gate or peer connection.** Nothing in this crate can
//! tell which kind of package declared an intent, which is the test of whether it is
//! `OD-PACKAGE-003`'s generic mechanism or an installer wearing a generic name. A second
//! package kind that declares intents needs no change here.
//!
//! **No source resolution beyond a file.** An intent's source is read as one file, relative
//! to the source root the caller names. A source naming a directory is refused as
//! unreadable rather than silently walked: placing a tree is a different operation with its
//! own conflict questions per contained path, and inventing it here would answer them by
//! default.
//!
//! **No ownership or scope inference.** Both classifications arrive on the intent, declared
//! by whatever placed the asset. `OD-PACKAGE-004` and `OD-PACKAGE-005` both refuse
//! self-attestation, so this crate never reads a class or a scope back out of the bytes it
//! is about to write.
//!
//! # Why this name
//!
//! `D-138`: a domain-neutral capability Nomos needs before XVPE can hold it is named as an
//! ordinary Nomos crate rather than reserved under a platform prefix, and migration is a
//! rename. Nothing in this crate's public contract carries Nomos vocabulary — no crate,
//! rule, finding or claim — so that rename would be mechanical.
//!
//! # Why the vocabulary is here rather than beside the manifest
//!
//! [`OwnershipClass`], [`PublicationScope`] and [`MaterializationIntent`] were
//! `nomos-integration-package`'s, and that crate's own doc named the condition for moving
//! them: `OD-CAPABILITY-002`'s rule is that a shared vocabulary earns a home below its
//! parties when a second party names it, and until a materializer existed the manifest was
//! the only party. It is no longer. They live here, below both the mechanism that consumes
//! an intent and every package kind that declares one, and `nomos-integration-package`
//! re-exports all three unchanged so no caller's spelling changed — the same shape
//! `nomos-ledger` re-exports `nomos-scope-verification`'s two primitives (`OD-LEDGER-037`)
//! and `nomos-lang-rust-package` re-exports `nomos-package`'s domains (`OD-PACKAGE-007`).

#![forbid(unsafe_code)]

mod materialization_error;
mod materialization_intent;
mod materialization_report;
mod materialization_roots;
mod materializer;
mod owned_region;
mod ownership_class;
mod placement;
mod placement_outcome;
mod publication_scope;
mod refusal;
mod target;

pub use materialization_error::MaterializationError;
pub use materialization_intent::MaterializationIntent;
pub use materialization_report::MaterializationReport;
pub use materialization_roots::MaterializationRoots;
pub use materializer::Materialize;
pub use owned_region::OwnedRegion;
pub use ownership_class::OwnershipClass;
pub use placement::Placement;
pub use placement_outcome::PlacementOutcome;
pub use publication_scope::PublicationScope;
pub use refusal::Refusal;
pub use target::{Escapes_The_Root, Is_Absolute};
