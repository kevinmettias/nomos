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
mod failure;
mod field_value;
mod normalize;
mod origin;
mod record;
mod render;
mod statement;
mod submission;
mod table;

pub use block::{BlockKind, Segment, SourceBlock};
pub use failure::{DecisionGap, Failure, Refusal, Severity};
pub use field_value::FieldValue;
pub use normalize::{ContentHash, HASH_PREFIX, Is_Normalized, Normalize};
pub use origin::Origin;
pub use record::{Parse_Record, Record, RecordError, RecordFrontMatter, RecordRelation};
pub use render::{Render_Record, RenderError, Round_Trips};
pub use statement::{NormativeStatement, StatementId, StatementKind};
pub use submission::{Submission, SubmissionKind, SubmissionState, Validate};
pub use table::{RowKind, TableDefect, TableRow, Table_Defects, Table_Rows};
