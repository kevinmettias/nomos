//! What an item was verified by, and what running it produced.
//!
//! The predicate is what the item declared; the record is what a run of it left behind.
//! Neither is read without the other -- an item carrying a predicate nothing ran and one
//! carrying a record no predicate declared are both incoherent -- so they are one module
//! rather than two files that happened to share a name prefix.


mod predicate;
mod record;

pub use predicate::VerificationPredicate;
pub use record::VerificationRecord;
