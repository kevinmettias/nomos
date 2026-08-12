//! What became of a normative statement recorded against this record.

use crate::normative_outcome::NormativeOutcome;

/// What became of a normative statement recorded against this record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormativeMovement
{
    pub statement_id: String,
    pub canonical_hash: String,
    pub outcome: NormativeOutcome,
}
