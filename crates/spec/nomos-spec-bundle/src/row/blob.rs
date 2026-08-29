//! One blob's bytes, addressed by the digest of what they are.

pub(crate) mod blob_encoding;

use serde::{Deserialize, Serialize};

use crate::BlobEncoding;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blob
{
    pub sha256: String,
    pub byte_length: i64,
    pub encoding: BlobEncoding,
    pub content: String,
}
