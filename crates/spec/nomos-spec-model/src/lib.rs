//! Band 1 — the specification content model.
//!
//! The single authority on what specification content hashes to. Every preservation
//! rule downstream compares a hash computed here against one recorded by v14, so a
//! disagreement in this crate makes the whole preservation ledger measure nothing.
//!
//! The algorithm was recovered from the v14 corpus, not chosen. See
//! `tests/normalizer_gate.rs`.
//!
//! # Why `check-crate-split` reports this crate, and why it stays one for now
//!
//! Its files fall into two groups that never reference each other: the content model --
//! blocks, statements, records, tables and how they render -- and the submission
//! vocabulary, which is a submission's state and kind together with the failures,
//! refusals, severities and field values it carries. The reading is correct. A submission
//! is *about* specification content and today names none of it, because the typed layer
//! it will be written through does not exist yet.
//!
//! That layer is open work rather than an omission, and a crate boundary drawn before the
//! writer exists would be drawn around a shape nothing has yet had to hold. Splitting is
//! also a band and ownership decision, which this repository settles in a record and not
//! in a commit that happened to be tidying the tree.

#![forbid(unsafe_code)]

mod block;
mod content_hash;
mod failure;
mod record;
mod render_error;
mod statement;
mod submission;
mod table;

// `Kind` would collide between `block::kind::Kind` and `statement::kind::Kind` if either
// flattened bare, so both keep their longer, table-naming public names here.
pub use block::kind::Kind as BlockKind;
pub use block::{Segment, SourceBlock};
pub use content_hash::{ContentHash, HASH_PREFIX, Is_Normalized, Normalize_Whitespace};
pub use failure::{DecisionGap, Failure, Refusal, Severity};
pub use record::error::Error;
pub use record::front_matter::FrontMatter;
pub use record::{Parse_Record, Record, RecordRelation};
pub use render_error::{Is_Round_Trip, Render_Record, RenderError};
pub use statement::id::Id;
pub use statement::kind::Kind as StatementKind;
pub use statement::NormativeStatement;
pub use submission::{FieldValue, Origin, Submission, SubmissionKind, SubmissionState, Validate_Submission};
pub use table::defect::Defect;
pub use table::row::Row;
pub use table::{RowKind, Table_Defects, Table_Rows};
