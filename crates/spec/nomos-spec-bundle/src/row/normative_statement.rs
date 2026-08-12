//! One normative statement, and the hash of the text it supersedes.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormativeStatement
{
    pub statement_id: String,
    pub node_id: String,
    pub kind: String,
    pub canonical_text: String,
    pub canonical_hash: String,
    pub supersedes_hash: Option<String>,
}
