//! Band 1 — the specification store.
//!
//! Three identifiers, never conflated: `node_id` is identity and travels in a bundle,
//! `uid` is a join surrogate and is never exported, `content_hash` is a content address.
//! Paths are navigation.

#![forbid(unsafe_code)]

mod authoring;
mod block_change;
mod claimed_record;
mod columns;
mod commit_report;
mod document_source;
mod edit_error;
mod edit_preview;
mod governing;
mod identity_change;
mod node_row;
mod node_summary;
mod normative_movement;
mod normative_outcome;
mod path_match;
mod read;
mod record;
mod record_projection;
mod record_write;
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
mod row_census;
mod row_scope;
mod rows;
mod schema;
mod staged_edit;
mod store;
mod store_error;
mod submission;
mod suite_authority;
mod table;
mod table_line;

pub use block_change::BlockChange;
pub use claimed_record::ClaimedRecord;
pub use commit_report::CommitReport;
pub use document_source::DocumentSource;
pub use edit_error::EditError;
pub use edit_preview::EditPreview;
pub use governing::{GOVERNING_RECORD_IDS, Seed_Governing_Records, SeedReport};
pub use identity_change::IdentityChange;
pub use node_row::NodeRow;
pub use node_summary::NodeSummary;
pub use normative_movement::NormativeMovement;
pub use normative_outcome::NormativeOutcome;
pub use path_match::PathMatch;
pub use record::{Disposition, Kind_Label};
pub use record_projection::RecordProjection;
pub use record_write::RecordWrite;
pub use row_census::RowCensus;
pub use row_scope::RowScope;
pub use schema::{Latest_Version, Migration, MIGRATIONS};
pub use staged_edit::StagedEdit;
pub use store::{AUTHORED, EXTERNAL, SpecificationStore};
pub use store_error::StoreError;
pub use submission::{Accept_Submission, AcceptError, Transport_Origin};
pub use suite_authority::SuiteAuthority;
pub use table::Table;
pub use table_line::TableLine;
