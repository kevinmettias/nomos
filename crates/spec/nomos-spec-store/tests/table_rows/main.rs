//! P3-ROWS. Table rows as typed subjects.
//!
//! The block stays the preservation authority — it holds the verbatim text and both v14
//! hashes, and P1-GATE proves those reproduce. Rows exist so a loss report can name what
//! went missing by identity rather than by count, which is the plan's own stated bar.
//!
//! Typing rows rather than discarding separators is what keeps `282` and `258` two
//! queries over one table. Neither is bent to match the other, and OD-SPEC-002 records
//! why the two numbers differ.
//!
//! P3-ROW-LINEAGE adds the third kind. A header is authored text that is not a datum, so
//! `282`, `258` and `234` are three queries rather than one number and two subtractions —
//! and a concept minted from a row traces to that row and not to the block around it.
//!
//! # One module per claim
//!
//! [`corpus`] is the only module that reads a corpus, and it holds both of this suite's
//! corpus-gated tests beside the root that gates them. That is not a convenience: the
//! derivation behind `tests/contract/tests/corpus_gates.rs` resolves a test to the corpora
//! it reaches through the helpers it calls *within one file*, so a gated test separated
//! from `Corpus_Root` stops being counted and the declared size of the hole silently drops.

mod census;
mod common;
mod corpus;
mod hashing;
mod lineage;
mod migration;
mod refusals;
