//! Band 1 — the specification store.
//!
//! Three identifiers, never conflated: `node_id` is identity and travels in a bundle,
//! `uid` is a join surrogate and is never exported, `content_hash` is a content address.
//! Paths are navigation.

#![forbid(unsafe_code)]

mod authoring;
mod columns;
mod governing;
mod read;
mod record;
/// The registration reader, which the library itself never calls.
///
/// `build.rs` reaches this file with `#[path]` and turns the registration directory into the
/// two tables `governing.rs` includes, so at run time the crate consumes the tables and not
/// the reader. Declared under `cfg(test)` for three reasons that meet at this line:
/// `Test_Every_Source_File_Should_Be_Reachable` refuses an undeclared file under `src/`; an
/// unconditional `mod` would make every item in it dead code under `-D warnings`; and the
/// reader's *refusals* are the whole of its value, so they have to be executed rather than
/// asserted. Under `cfg(test)` they run with `cargo test -p nomos-spec-store`.
#[cfg(test)]
mod registration;
mod rows;
mod schema;
mod store;
mod submission;

pub use authoring::{
    BlockChange, ClaimedRecord, CommitReport, EditError, EditPreview, IdentityChange, NormativeMovement,
    NormativeOutcome, RecordProjection, RecordWrite, StagedEdit,
};
pub use governing::{GOVERNING_RECORD_IDS, Seed_Governing_Records, SeedReport};
pub use read::{DocumentSource, NodeSummary, PathMatch, TableLine};
pub use record::{Disposition, Kind_Label};
pub use rows::{RowCensus, RowScope};
pub use schema::{Latest_Version, Migration, MIGRATIONS};
pub use store::{
    AUTHORED, EXTERNAL, NodeRow, SpecificationStore, StoreError, SuiteAuthority, Table,
};
pub use submission::{Accept_Submission, AcceptError, Transport_Origin};
