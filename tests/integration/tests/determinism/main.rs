//! Every declared [`Strategy`] in this workspace, checked against the domain that
//! declared it.
//!
//! # What was wrong before this file existed
//!
//! `nomos-contracts` defined `Strategy`, `DeterminismStrength`, `ReproducibilityScope`
//! and `TraceEquivalence`, wrote a six-row table naming which domain claims what, and
//! nothing in the workspace implemented any of it. That crate is the one every non-Rust
//! peer reimplements, so an unimplemented declaration there is not an unused type — it is
//! a published protocol commitment that no implementation has ever been held to.
//!
//! The properties were not absent, only the declarations. `nomos-lang-rust` has read the
//! same corpus twice and agreed with itself since it was written, and `nomos-workspace`
//! has produced byte-identical snapshots across a hundred ingestion orders. Both of those
//! assertions are corpus-gated and neither has ever run in CI.
//!
//! # The shape of every test here
//!
//! One test per declared domain, and each one:
//!
//! 1. hands [`Verify`] the domain's own `Strategy` and a closure that produces its bytes,
//!    which discharges coherence and repetition at the declared strength;
//! 2. asks [`Cross_Environment_Owed`] what the declared *scope* additionally requires and
//!    discharges exactly that — a second process, and where the scope reaches
//!    `CrossPlatform` or above, agreement with a digest committed to this tree.
//!
//! Step 2 is deliberately not a list of things to remember. It is a function of the
//! declaration, so raising a scope raises the obligation with no second edit, and a domain
//! that cannot meet its declared scope fails here rather than in review.
//!
//! [`Strategy`]: nomos_contracts::Strategy
//! [`Verify`]: nomos_integration_tests::Verify
//! [`Cross_Environment_Owed`]: nomos_integration_tests::Cross_Environment_Owed


mod harness;
mod controls;
mod declarations;
mod domains;
mod goldens;
mod productions;
mod spec_productions;
