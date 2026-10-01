//! What the materializer does to a real tree, one ownership class at a time.
//!
//! `OD-PACKAGE-004` names three behaviours and this suite is organised by them, because they
//! are the acceptance criteria rather than a grouping chosen here: a `UserOwned` target is
//! never written, a `GeneratedOwned` one is overwritten unconditionally, and a `Composed`
//! one has only its owned region written. The two remaining modules are the properties that
//! cut across all three — that a failed run leaves the tree as it found it, and that the
//! publication scope riding alongside gates none of it.
//!
//! `rows` is the one module named after a shape rather than a class. `OD-PACKAGE-004`
//! version 3 names adding a row to a table as the corner where a `Composed` target's two
//! regions meet, so it is exercised as its own case rather than folded into `composed`,
//! whose fixtures are about locating a region at all.
//!
//! Every test runs through a substituted `nomos_platform::FileSystem`, not because a real
//! one would be slow but because the atomicity claim is about a write failing partway
//! through a run, and no real filesystem can be asked to fail the second of three writes.

mod atomicity;
mod composed;
mod fake;
mod fixtures;
mod generated_owned;
mod rows;
mod scope;
mod user_owned;
