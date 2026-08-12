//! Phase 0 acceptance: the ledger actually excludes.
//!
//! Every test here has a negative control, because a guard that has never been watched
//! failing is not a guard — it is a test that would pass just as happily if the thing it
//! checks were deleted.

// folder-organization: coherent: one module per thing the ledger has to exclude, and the
// list is the acceptance criteria rather than a grouping chosen here. Every module is a
// sibling claim about the same guard, each with its own negative control, so a subsystem
// folder over any subset of them would assert a relationship between those claims that
// nothing decides — and the reader who wants to know what this suite covers reads exactly
// this list.

mod claiming;
mod common;
mod concurrency;
mod dependencies;
mod finishing;
mod interleaving;
mod lapse;
mod persistence;
mod records;
mod takeover;
mod validation;
