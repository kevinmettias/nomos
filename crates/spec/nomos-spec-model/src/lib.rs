//! Band 1 — the specification content model.
//!
//! The single authority on what specification content hashes to. Every preservation
//! rule downstream compares a hash computed here against one recorded by v14, so a
//! disagreement in this crate makes the whole preservation ledger measure nothing.
//!
//! The algorithm was recovered from the v14 corpus, not chosen. See
//! `tests/normalizer_gate.rs`.

#![forbid(unsafe_code)]

mod block;
mod block_kind;
mod decision_gap;
mod failure;
mod field_value;
mod normalize;
mod origin;
mod record;
mod record_error;
mod record_front_matter;
mod record_relation;
mod refusal;
mod render;
mod row_kind;
mod severity;
mod statement;
mod statement_id;
mod statement_kind;
mod submission;
mod submission_kind;
mod submission_state;
mod table;
mod table_defect;
mod table_row;

pub use block::{Segment, SourceBlock};
pub use block_kind::BlockKind;
pub use decision_gap::DecisionGap;
pub use failure::Failure;
pub use field_value::FieldValue;
pub use normalize::{ContentHash, HASH_PREFIX, Is_Normalized, Normalize};
pub use origin::Origin;
pub use record::{Parse_Record, Record};
pub use record_error::RecordError;
pub use record_front_matter::RecordFrontMatter;
pub use record_relation::RecordRelation;
pub use refusal::Refusal;
pub use render::{Render_Record, RenderError, Round_Trips};
pub use row_kind::RowKind;
pub use severity::Severity;
pub use statement::NormativeStatement;
pub use statement_id::StatementId;
pub use statement_kind::StatementKind;
pub use submission::{Submission, Validate};
pub use submission_kind::SubmissionKind;
pub use submission_state::SubmissionState;
pub use table::{Table_Defects, Table_Rows};
pub use table_defect::TableDefect;
pub use table_row::TableRow;
