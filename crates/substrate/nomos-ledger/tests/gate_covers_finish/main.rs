//! An item cannot be finished by a predicate that checks less than the gate does.
//!
//! Every item's `verification` argv is a `cargo test` invocation. The gate lints first,
//! with `-D warnings`, so an item could be recorded verified with the gate already red.
//! `P9-SKIP` and `P9-PHASE-GAP` were both finished that way and caught afterwards by a
//! person running clippy by hand — which is the arrangement the ledger exists to replace.
//!
//! Those two instances cannot be replayed from git, and that is worth stating precisely.
//! Clippy passes on both commits (`26566c6` and `31f4450`, measured, with a confirmed-red
//! control). The defect lived in the working tree between `work finish` and `git commit`
//! — 180 and 185 seconds respectively — and was repaired before the commit was written.
//! So the instances are reproduced here in the shape they actually had: a predicate that
//! exits zero while the gate's own step does not.
//!
//! Every test has a negative control, because a guard that has never been watched failing
//! is not a guard.


mod launcher;
mod derivation;
mod ordering;
mod red_gate;
mod repository_gate;
mod unknown;
