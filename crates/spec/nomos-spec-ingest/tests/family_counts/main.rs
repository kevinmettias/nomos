//! P3-COUNTS. Every restored family measured, with the definition that produced it.
//!
//! The register at `tests/corpus/families/counts.json` is the single home for these
//! numbers. Each entry carries what it counted, how it was extracted and what it was
//! measured over, and where the plan states a different figure the plan's is recorded as
//! superseded rather than quietly replaced — D-132 makes that the rule rather than a
//! habit, and OD-SPEC-002 already settled the first two instances of it.
//!
//! A definition written only in prose is a definition nothing checks, so every entry has
//! an extractor here and an entry with none fails. The extractor is the definition; the
//! sentence in the register is its reading.
//!
//! # How the modules divide
//!
//! [`structure`] keeps the register honest about itself and needs no corpus.
//! [`measurement`] re-measures every entry against the corpus, and holds both corpus-gated
//! tests beside the two roots that gate them — `tests/contract/src/gates.rs` follows calls
//! within one file's text, so a gated test in a sibling module would be counted by nobody.
//! The extractors are the definitions, and they divide by what they read: [`volumes`] for a
//! volume's tables, rows and code, [`headings`] for the ways a section is addressed, and
//! [`catalog`] for the two figures taken over something other than the volumes.

mod catalog;
mod register;
mod headings;
mod measurement;
mod structure;
mod volumes;
