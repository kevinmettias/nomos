//! Phase 0 acceptance: the ledger actually excludes.
//!
//! Every test here has a negative control, because a guard that has never been watched
//! failing is not a guard — it is a test that would pass just as happily if the thing it
//! checks were deleted.


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
