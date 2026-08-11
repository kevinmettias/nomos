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
mod normalize;
mod record;
mod render;
mod statement;
mod submission;
mod table;

pub use block::{BlockKind, Segment, SourceBlock};
pub use normalize::{ContentHash, HASH_PREFIX, Is_Normalized, Normalize};
pub use record::{Parse_Record, Record, RecordError, RecordFrontMatter, RecordRelation};
pub use render::{Render_Record, RenderError, Round_Trips};
pub use statement::{NormativeStatement, StatementId, StatementKind};
pub use submission::{
    DecisionGap, Failure, FieldValue, Origin, Refusal, Severity, Submission, SubmissionKind, SubmissionState, Validate,
};
pub use table::{RowKind, Table_Defects, Table_Rows, TableDefect, TableRow};
