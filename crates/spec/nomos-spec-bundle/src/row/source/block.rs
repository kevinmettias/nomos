//! One authored block of a source document, with both v14 hashes.

use serde::{Deserialize, Serialize};

use crate::DocumentRef;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block
{
    pub document: DocumentRef,
    pub ordinal: i64,
    pub kind: String,
    pub heading_path: String,
    pub text: String,
    pub content_hash: String,
    pub normalized_hash: String,
}
