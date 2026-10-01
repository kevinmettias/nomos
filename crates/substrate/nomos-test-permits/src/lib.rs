//! Zone: Substrate — the bound a test process admits its compiler databases through, and the two
//! tests that falsify it.
//!
//! libtest runs every test of a crate as a thread of one process, one per core, so a suite whose
//! tests each load a whole compiler database loads all of them at once. [`JudgmentPermits`] is the
//! counting semaphore such a suite admits them through. It was written for
//! `nomos-gate-orchestration`'s suite and then copied byte for byte into
//! `nomos-check-orchestration`'s, because neither crate could reach the other's test module. It is
//! here so that both use one copy, and a third suite takes it rather than copying it again --
//! which `nomos-api`'s did, when its gate-response tests became the third suite to hold a compiler
//! database per test thread.
//!
//! # Only the mechanism is here
//!
//! No bound is. Each suite admits a different unit -- a judgment over the repository's root in
//! gate-orchestration, one compiler-backed provider call in check-orchestration, and every gate
//! judgment a test makes through nomos-api's handlers, a run or a comparison -- and derives its
//! limit from its own suite's measurement, so each keeps its own constant with the derivation
//! beside it. One number here would be a figure measured for one suite and imposed on another.
//!
//! # Why Substrate, and why only as a dev-dependency
//!
//! Two consumers, the two orchestration crates, sit in Application Service, and the third,
//! `nomos-api`, sits in Host. `nomos-architecture.json` permits both zones to depend on Substrate,
//! so each names this crate without an exception. This crate names nothing but the standard
//! library, so it asks nothing of Substrate's own permits. It is a dev-dependency of each of the
//! three and of nothing else: no production crate depends on it, and nothing it exports reaches a
//! shipped build.

#![forbid(unsafe_code)]

mod judgment_permits;

pub use judgment_permits::JudgmentPermits;
