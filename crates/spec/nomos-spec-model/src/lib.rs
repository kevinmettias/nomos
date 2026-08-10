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
mod table;

pub use block::{BlockKind, Segment, SourceBlock};
pub use normalize::{ContentHash, HASH_PREFIX, Is_Normalized, Normalize};
pub use record::{Parse_Record, Record, RecordError, RecordFrontMatter, RecordRelation};
pub use render::{RenderError, Render_Record, Round_Trips};
pub use statement::{NormativeStatement, StatementId, StatementKind};
pub use table::{RowKind, TableDefect, TableRow, Table_Defects, Table_Rows};
