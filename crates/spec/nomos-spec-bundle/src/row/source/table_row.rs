//! One pipe line of a table, addressed by the block that carries it.

use serde::{Deserialize, Serialize};

use crate::OrdinalRef;

/// One pipe line of a table, addressed by the block that carries it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableRow
{
    pub block: OrdinalRef,
    pub ordinal: i64,
    pub table_ordinal: i64,
    pub kind: String,
    pub cells: Vec<String>,
    pub text: String,
    pub content_hash: String,
    pub normalized_hash: String,
}
