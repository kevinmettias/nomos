//! An edit to the store: what it stages, what it would change, and what committing it
//! reported.
//!
//! Six files at the crate root spelled this relationship in their name prefixes --
//! `edit_`, `staged_`, `_change`, `commit_` -- and a reader had to reassemble it from
//! those. The transaction is one subject and this is where it lives.

pub(crate) mod normative_movement;
pub(crate) mod normative_outcome;

pub(crate) mod block_change;
pub(crate) mod commit_report;
pub(crate) mod error;
pub(crate) mod identity_change;
pub(crate) mod preview;
pub(crate) mod staged;
