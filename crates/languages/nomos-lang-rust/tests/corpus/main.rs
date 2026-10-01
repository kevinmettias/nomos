//! The provider against a corpus it did not write.
//!
//! Everything in `tests/guarantee.rs` is a sample chosen by the person asserting the
//! property, which is the weakest possible evidence that a reader works: the samples were
//! written by somebody who already knew what the reader did. This suite points it at
//! `F:/repos/xvpe` — several thousand files of somebody else's Rust, none of it written
//! with this provider in mind.
//!
//! # Every number here comes with what it was measured over
//!
//! A corpus report that says "3 files failed" and not "3 of 7604" is a number nobody can
//! calibrate, and one that says "0 findings" without saying how many files it opened is
//! the shape of a check that walked nothing and reported clean. Every assertion below
//! either names its denominator or fails.
//!
//! # Opt-in by path, and loud when configured and unreadable
//!
//! `NOMOS_RUST_CORPUS` overrides the root. When it is set and unreadable the tests fail,
//! because a gate that quietly skips its own subject is worse than one that fails. When
//! it is unset and the default root is absent — any machine that is not this one — the
//! tests return, having asserted nothing and said so. This is the same bargain
//! `nomos-spec-model`'s `normalizer_gate` strikes with `NOMOS_V14_CORPUS`.
//!
//! Reading these files is not a dependency on the sibling workspace. D-130 governs what
//! Nomos may *build against*, and this crate builds against nothing there; it reads text.
//!
//! # Why every test is in one module
//!
//! All six of this suite's tests are corpus-gated, so all six are in [`claims`] beside the
//! root that gates them. `tests/contract/src/gates.rs` resolves a test to the corpora it
//! reaches by following calls **within one file**, and none of these tests names the
//! variable itself — each reaches it through `Corpus_Or_Skip`. A test moved to a sibling
//! module for balance would be gated in fact and counted by nobody, which would drop the
//! declared size of the hole `docs/records/OD-GATE-001` measures. What could be moved is the
//! machinery: the walk, what a walk found, and how a reported name is checked against the
//! file it came from.

mod claims;
mod soundness;
mod walk;
mod walked;
