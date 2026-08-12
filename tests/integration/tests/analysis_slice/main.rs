//! The vertical slice.
//!
//! Every crate below this one passes its own tests over inputs its author chose. What
//! none of them can answer is whether the pieces compose — whether the key
//! `nomos-lang-rust` writes is the key `nomos_analysis::Reader` looks up, whether a fact
//! resolved through the registry is the fact the provider produced, whether invalidation
//! reaches a derived fact along an edge nobody declared by hand. Each is an agreement
//! between two crates, and an agreement is exactly what neither party can verify alone.
//!
//! # The three things the prototype learned expensively
//!
//! A rule firing thousands of times is background hum, and reporting it as a result
//! trains everyone to ignore the report. A check that found zero findings corpus-wide was
//! broken rather than satisfied, every time. And a negative control that has never failed
//! is not a control — it is a test whose failure mode nobody has observed.
//!
//! Each has a test here, and each is stated over its denominator.
//!
//! # How the modules divide
//!
//! By the claim, except for one module that is divided by the corpus instead. [`precision`]
//! names every fact a change recomputes, over the six-file corpus small enough to know
//! entirely. [`context`] asserts that no component of a fact's identity was invented.
//! [`providers`] is two providers of one capability, and [`spending`] is what a lowered
//! floor buys per subject. [`lessons`] carries the three the prototype learned expensively.
//!
//! [`scale`] is the exception, and it is not an arrangement anybody chose. All four
//! corpus-gated tests are there beside `Scale_Corpus_Or_Skip`, the root that gates them,
//! because `tests/contract/src/gates.rs` resolves a test to the corpora it reaches by
//! following calls **within one file** and none of these tests names `NOMOS_RUST_CORPUS`
//! itself. A gated test left among the claims it belongs with would be gated in fact and
//! counted by nobody, which is the silence `tests/contract/tests/corpus_gates.rs` exists to
//! measure. Two of the four belong to [`providers`] and [`lessons`] by subject, and each says
//! where it came from at its own definition.

mod common;
mod context;
mod lessons;
mod precision;
mod providers;
mod scale;
mod spending;
