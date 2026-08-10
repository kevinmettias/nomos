//! Band 1 — the specification store.
//!
//! Three identifiers, never conflated: `node_id` is identity and travels in a bundle,
//! `uid` is a join surrogate and is never exported, `content_hash` is a content address.
//! Paths are navigation.

#![forbid(unsafe_code)]

mod authoring;
mod governing;
mod read;
mod record;
mod rows;
mod schema;
mod store;

pub use authoring::{
    BlockChange, ClaimedRecord, CommitReport, EditError, EditPreview, IdentityChange,
    NormativeMovement, NormativeOutcome, RecordProjection, RecordWrite, StagedEdit,
};
pub use governing::{GOVERNING_RECORD_IDS, Seed_Governing_Records, SeedReport};
pub use read::{DocumentSource, NodeSummary, PathMatch, TableLine};
pub use record::{Disposition, Kind_Label, Kind_Of};
pub use rows::{RowCensus, RowScope};
pub use schema::{Latest_Version, MIGRATIONS, Migration};
pub use store::{AUTHORED, EXTERNAL, SpecificationStore, StoreError, Table};

/// The registration reader, which the library itself never calls.
///
/// `build.rs` reaches this file with `#[path]` and turns the registration directory into the
/// two tables `governing.rs` includes, so at run time the crate consumes the tables and not
/// the reader. Declared under `cfg(test)` for three reasons that meet at this line:
/// `Test_Every_Source_File_Should_Be_Reachable` refuses an undeclared file under `src/`; an
/// unconditional `mod` would make every item in it dead code under `-D warnings`; and the
/// reader's *refusals* are the whole of its value, so they have to be executed rather than
/// asserted. Under `cfg(test)` they run with `cargo test -p nomos-spec-store`.
///
/// **Last in the file, and that is not tidiness.** `Without_Test_Modules` in
/// `tests/contract/src/gates.rs` blanks from a `#[cfg(test)]` marker to the *next* brace
/// block, whichever block that turns out to be. A `#[cfg(test)]` on an item that carries no
/// braces — this `mod` declaration — therefore eats whatever braced item follows it. Placed
/// among the other `mod` lines it swallowed `pub use authoring::{…}`, and
/// `Test_Every_Crates_Public_Surface_Should_Match_Its_Snapshot` reported eleven exports
/// missing and an unresolvable re-export. Nothing braced follows it here, so the scan finds
/// no block and stops. Anything braced added below this line will disappear from the
/// snapshot; add it above.
#[cfg(test)]
mod registration;
