//! One value of one field of a submission, with the origin `OD-SPEC-010` requires it to carry.

use serde::{Deserialize, Serialize};

/// One value of one field of a submission, with the origin `OD-SPEC-010` requires it to carry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Value
{
    pub node_id: String,
    pub field: String,
    pub ordinal: i64,
    pub origin: String,
    pub value: String,
    pub value_hash: String,
    pub supersedes_hash: Option<String>,
    pub recorded_at: String,
}
