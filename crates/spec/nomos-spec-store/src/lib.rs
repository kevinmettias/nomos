//! Band 1 — the specification store.
//!
//! Three identifiers, never conflated: `node_id` is identity and travels in a bundle,
//! `uid` is a join surrogate and is never exported, `content_hash` is a content address.
//! Paths are navigation.

#![forbid(unsafe_code)]

mod addressing;
mod authoring;
mod edit;
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
mod migration;
mod store;
mod submission;
mod table;

pub use addressing::{DocumentPath, DocumentRevision};
pub use edit::block_change::BlockChange;
pub use record::claimed_record::ClaimedRecord;
pub use edit::commit_report::CommitReport;
pub use read::document_source::DocumentSource;
pub use edit::error::Error as EditError;
pub use edit::preview::Preview as EditPreview;
pub use edit::identity_change::IdentityChange;
pub use read::node_row::NodeRow;
pub use read::node_summary::NodeSummary;
pub use edit::normative_movement::NormativeMovement;
pub use edit::normative_outcome::NormativeOutcome;
pub use read::path_match::PathMatch;
pub use record::{Disposition, Kind_Label};
pub use record::projection::Projection as RecordProjection;
pub use record::write::Write;
pub use table::row_census::RowCensus;
pub use table::row_scope::RowScope;
pub use migration::{Latest_Version, Migration, MIGRATIONS};
pub use edit::staged_edit::StagedEdit;
pub use store::{
    AUTHORED, EXTERNAL, GOVERNING_RECORD_IDS, SeedReport,
    Seed_Governing_Records, SpecificationStore,
};
pub use store::relation::{
    Constraint, FromNodeId, InverseRelationType, Tier, ToNodeId, TypeName,
};
pub use store::error::Error as StoreError;
pub use submission::{Accept_Submission, AcceptError, Transport_Origin};
pub use table::suite_authority::{SuiteAuthority, SuiteId, SuiteTitle};
pub use table::Table;
pub use table::line::Line as TableLine;
