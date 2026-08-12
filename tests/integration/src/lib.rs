//! Band 100 — the vertical slice.
//!
//! A peer of `nomos-contract-tests`, not a layer above or below it. That crate observes
//! the workspace's shape through cargo metadata and compiles against almost nothing; this
//! one drives the product through every seam it has. Both are terminal, neither may name
//! the other, and giving the observer a dependency on a language provider would have made
//! it a participant in what it is supposed to be watching from outside.
//!
//! # What a slice is for
//!
//! Not coverage. Every crate below here passes its own tests over inputs its author
//! chose, and the question none of those tests can answer is whether the pieces compose:
//! whether the key `nomos-lang-rust` writes is the key `nomos_analysis::Reader` looks up,
//! whether a fact read through the registry is the fact the provider produced, whether
//! invalidation reaches a derived fact through a dependency edge nobody declared by hand.
//! Each of those is an agreement between two crates that neither crate can test alone.
//!
//! # Two corpora, because they answer different questions
//!
//! `F:/repos/xvpe` answers whether this works at scale — 7,500 files nobody wrote for
//! this test. It cannot answer the precision question, because naming the exact set of
//! facts a change should recompute requires knowing the whole dependency graph by hand.
//!
//! `tests/corpus/analysis` is small enough to know entirely. It is where "recomputes
//! exactly its descendants and no more" is asserted by *naming* them, which is the only
//! form of that assertion worth making: a count of two is satisfied by recomputing the
//! wrong two.
//!
//! # Why `check-crate-split` reports this crate, and why the answer is no
//!
//! Its files fall into two groups that never reference each other: the slice and its
//! readings, and the four types describing what one pass did. Both readings are true, and
//! neither is an argument for a second crate. This crate's consumers are the test binaries
//! beside it and nothing else -- it is published nowhere, pinned by nobody, and depended on
//! by no version -- so the question that check exists to ask, whether somebody would want a
//! subset of this and not the rest, has one answer here and it is no.

#![forbid(unsafe_code)]

mod composition;
mod context;
mod corpus;
mod determinism;
mod run_report;
mod slice;
mod surface;
mod verification;

pub use composition::Registered;
pub use context::{
    Configuration_Rendering, Host_Variant, Resolved_Configuration, CONFIGURATION_SCHEMA,
};
pub use corpus::{Corpus, Name_Keys, SourceFile, Subject_Of_Path, Walk};
pub use determinism::{Child_Variable, Digest_In, Report_Line};
pub use run_report::{Edited, Recompute, Resolved, RunReport};
pub use slice::Slice;
pub use verification::{Approximate_Floor, CrossEnvironment, Cross_Environment_Owed, Parsed_Floor, Production, Verification, Verify};
pub use surface::{
    Decode_Surface, Public_Items, Surface, CAPABILITY as SURFACE_CAPABILITY,
};
