//! The cross-revision regression report, and the register that stands behind it.
//!
//! Two halves that check each other. [`register`] asks whether the checked-in headline
//! register is internally coherent — every clause of the plan answered exactly once, every
//! restored family carrying one fate entry, every quoted figure present in the counts
//! register. That runs anywhere. [`archives`] asks whether the register's numbers reproduce
//! from the revision archives themselves, which needs the archives.
//!
//! # Why every archive claim is in one module
//!
//! Nine of the fourteen tests are corpus-gated and all nine are in [`archives`] beside
//! `Archives`, the root that gates them. `tests/contract/src/gates.rs` resolves a test to
//! the corpora it reaches by following calls **within one file**, and none of these tests
//! names `NOMOS_SPEC_ARCHIVES` itself. A gated test moved to a sibling module would be
//! gated in fact and counted by nobody, which is the silence
//! `tests/contract/tests/corpus_gates.rs` exists to measure.

mod archives;
mod rows;
mod measure;
mod register;
