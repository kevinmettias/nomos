//! What an item was verified by, and what running it produced.
//!
//! The predicate is what the item declared; the record is what a run of it left behind.
//! Neither is read without the other -- an item carrying a predicate nothing ran and one
//! carrying a record no predicate declared are both incoherent -- so they are one module
//! rather than two files that happened to share a name prefix.
//!
//! [`VerificationPredicate`] itself is not declared here: `OD-LEDGER-037` found it
//! ledger-agnostic and `nomos-scope-verification` is where it now lives, alongside
//! [`Territory`](crate::Territory). This module keeps the record half, which genuinely is
//! the ledger's own -- what a run of a predicate left behind against one held item.

#[path = "verification/verification_record.rs"]
mod record;

pub use nomos_scope_verification::VerificationPredicate;
pub use record::VerificationRecord;
