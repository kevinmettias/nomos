//! The two properties the item is done when.
//!
//! A portable snapshot over `F:/repos/xvpe` interpretable by a process with no access to
//! that tree, and 100 ingestion-order permutations that yield byte-identical queries.
//!
//! # Opt-in by path, loud when configured and unreadable
//!
//! `NOMOS_RUST_CORPUS` overrides the root. Set and unreadable is a failure; absent and
//! unconfigured returns having asserted nothing and said so. The same bargain
//! `nomos-spec-model`'s `normalizer_gate` strikes with `NOMOS_V14_CORPUS`.
//!
//! # Why the corpus tests share one module
//!
//! Five of this suite's six tests are corpus-gated, and all five are in
//! [`over_the_corpus`] beside the root that gates them. The derivation behind
//! `tests/contract/tests/corpus_gates.rs` resolves a test to the corpora it reaches by
//! following calls **within one file** — none of these tests names the variable itself — so
//! a gated test in a sibling module would be counted by nobody and the declared size of the
//! hole would quietly drop. The sixth test needs no corpus and lives with the permutation it
//! is about.

mod arrival;
mod over_the_corpus;
mod permutation;
mod walk;
