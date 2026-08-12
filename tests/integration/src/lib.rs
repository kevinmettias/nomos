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

#![forbid(unsafe_code)]

mod context;
mod corpus;
mod cross_environment;
mod determinism;
mod edited;
mod floors;
mod production;
mod recompute;
mod resolved;
mod run_report;
mod slice;
mod source_file;
mod surface;
mod verification;

pub use context::{
    Configuration_Rendering, Host_Variant, Resolved_Configuration, CONFIGURATION_SCHEMA,
};
pub use corpus::{Corpus, Subject_Of_Path, Walk};
pub use cross_environment::{Cross_Environment_Owed, CrossEnvironment};
pub use determinism::{Child_Variable, Digest_In, Report_Line};
pub use edited::Edited;
pub use floors::{Approximate_Floor, Parsed_Floor};
pub use production::Production;
pub use recompute::Recompute;
pub use resolved::Resolved;
pub use run_report::RunReport;
pub use slice::Slice;
pub use source_file::SourceFile;
pub use verification::{Verification, Verify};
pub use surface::{
    Decode_Surface, Public_Items, Surface, CAPABILITY as SURFACE_CAPABILITY,
};
