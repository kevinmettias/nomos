//! One source document at one revision, and the blob holding its bytes.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceDocument
{
    pub path: String,
    pub revision: String,
    pub blob_sha256: String,
}
